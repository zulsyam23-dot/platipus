use std::collections::BTreeMap;
use std::fmt;

use crate::StateKind;

/// A run time value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    List(Vec<Value>),
    Map(BTreeMap<String, Value>),
}

impl Value {
    pub fn int(value: i64) -> Self {
        Value::Int(value)
    }

    pub fn float(value: f64) -> Self {
        Value::Float(value)
    }

    pub fn text(value: impl Into<String>) -> Self {
        Value::Text(value.into())
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(value) => Some(*value),
            Value::Int(value) => Some(*value as f64),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(value) => Some(*value),
            Value::Float(value) => Some(*value as i64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Value::Text(value) => Some(value.as_str()),
            _ => None,
        }
    }

    pub fn truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(value) => *value,
            Value::Int(value) => *value != 0,
            Value::Float(value) => *value != 0.0,
            Value::Text(value) => !value.is_empty(),
            Value::List(items) => !items.is_empty(),
            Value::Map(entries) => !entries.is_empty(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => f.write_str("null"),
            Value::Bool(value) => write!(f, "{value}"),
            Value::Int(value) => write!(f, "{value}"),
            Value::Float(value) => write!(f, "{value}"),
            Value::Text(value) => write!(f, "{value}"),
            Value::List(items) => {
                f.write_str("[")?;
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{item}")?;
                }
                f.write_str("]")
            }
            Value::Map(entries) => {
                f.write_str("{")?;
                for (index, (key, value)) in entries.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{key}: {value}")?;
                }
                f.write_str("}")
            }
        }
    }
}

/// Which storage a value lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Scope {
    Instance,
    Shared,
    Global,
    Persistent,
}

impl Scope {
    pub const fn of(kind: StateKind) -> Option<Self> {
        match kind {
            StateKind::Local => Some(Scope::Instance),
            StateKind::Shared => Some(Scope::Shared),
            StateKind::Global => Some(Scope::Global),
            StateKind::Persistent => Some(Scope::Persistent),
            StateKind::Derived => None,
        }
    }
}

type Subscriber = Box<dyn Fn(&Value)>;

struct Cell {
    value: Value,
    subscribers: Vec<(u64, Subscriber)>,
    next_token: u64,
}

/// Reactive storage for the values a program declares.
#[derive(Default)]
pub struct StateStore {
    cells: BTreeMap<(Scope, String), Cell>,
    persistent: BTreeMap<String, Value>,
}

impl fmt::Debug for StateStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StateStore")
            .field("values", &self.cells.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl StateStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn declare(&mut self, scope: Scope, name: &str, initial: Value) {
        let restored = match scope {
            Scope::Persistent => self
                .persistent
                .get(name)
                .cloned()
                .unwrap_or(initial),
            _ => initial,
        };
        self.cells.insert(
            (scope, name.to_string()),
            Cell {
                value: restored,
                subscribers: Vec::new(),
                next_token: 0,
            },
        );
    }

    pub fn get(&self, scope: Scope, name: &str) -> Option<&Value> {
        self.cells.get(&(scope, name.to_string())).map(|cell| &cell.value)
    }

    /// Writes a value and notifies subscribers when it actually changed.
    pub fn set(&mut self, scope: Scope, name: &str, next: Value) -> bool {
        let Some(cell) = self.cells.get_mut(&(scope, name.to_string())) else {
            return false;
        };
        if cell.value == next {
            return false;
        }
        cell.value = next.clone();
        if scope == Scope::Persistent {
            self.persistent.insert(name.to_string(), next.clone());
        }
        for (_, subscriber) in &cell.subscribers {
            subscriber(&next);
        }
        true
    }

    pub fn contains(&self, scope: Scope, name: &str) -> bool {
        self.cells.contains_key(&(scope, name.to_string()))
    }

    /// Registers a subscriber and returns a token that removes it again.
    pub fn subscribe(
        &mut self,
        scope: Scope,
        name: &str,
        subscriber: impl Fn(&Value) + 'static,
    ) -> Option<u64> {
        let cell = self.cells.get_mut(&(scope, name.to_string()))?;
        cell.next_token += 1;
        let token = cell.next_token;
        cell.subscribers.push((token, Box::new(subscriber)));
        Some(token)
    }

    pub fn unsubscribe(&mut self, scope: Scope, name: &str, token: u64) -> bool {
        let Some(cell) = self.cells.get_mut(&(scope, name.to_string())) else {
            return false;
        };
        let before = cell.subscribers.len();
        cell.subscribers.retain(|(existing, _)| *existing != token);
        cell.subscribers.len() != before
    }

    pub fn names(&self) -> Vec<(Scope, String)> {
        self.cells.keys().cloned().collect()
    }

    /// Copies persisted values so a new store can restore them.
    pub fn restore_from(&mut self, other: &StateStore) {
        self.persistent = other.persistent.clone();
    }

    pub fn persisted(&self) -> &BTreeMap<String, Value> {
        &self.persistent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn stores_and_reads_values_per_scope() {
        let mut store = StateStore::new();
        store.declare(Scope::Instance, "count", Value::int(0));
        store.declare(Scope::Global, "total", Value::int(5));
        assert_eq!(store.get(Scope::Instance, "count"), Some(&Value::int(0)));
        assert_eq!(store.get(Scope::Global, "total"), Some(&Value::int(5)));
        assert_eq!(store.get(Scope::Instance, "missing"), None);
    }

    #[test]
    fn writing_an_unchanged_value_does_not_notify() {
        let mut store = StateStore::new();
        store.declare(Scope::Instance, "count", Value::int(1));
        let seen = Rc::new(RefCell::new(0));
        let observer = Rc::clone(&seen);
        store.subscribe(Scope::Instance, "count", move |_| {
            *observer.borrow_mut() += 1;
        });
        assert!(!store.set(Scope::Instance, "count", Value::int(1)));
        assert_eq!(*seen.borrow(), 0);
        assert!(store.set(Scope::Instance, "count", Value::int(2)));
        assert_eq!(*seen.borrow(), 1);
    }

    #[test]
    fn unsubscribing_stops_notifications() {
        let mut store = StateStore::new();
        store.declare(Scope::Instance, "count", Value::int(0));
        let seen = Rc::new(RefCell::new(0));
        let observer = Rc::clone(&seen);
        let token = store
            .subscribe(Scope::Instance, "count", move |_| {
                *observer.borrow_mut() += 1;
            })
            .expect("cell exists");
        store.set(Scope::Instance, "count", Value::int(1));
        assert!(store.unsubscribe(Scope::Instance, "count", token));
        store.set(Scope::Instance, "count", Value::int(2));
        assert_eq!(*seen.borrow(), 1);
    }

    #[test]
    fn setting_an_undeclared_value_is_rejected() {
        let mut store = StateStore::new();
        assert!(!store.set(Scope::Instance, "nope", Value::Null));
    }

    #[test]
    fn persistent_values_survive_a_redeclare() {
        let mut store = StateStore::new();
        store.declare(Scope::Persistent, "token", Value::text("a"));
        store.set(Scope::Persistent, "token", Value::text("b"));
        let mut next = StateStore::new();
        next.restore_from(&store);
        next.declare(Scope::Persistent, "token", Value::text("a"));
        assert_eq!(next.get(Scope::Persistent, "token"), Some(&Value::text("b")));
    }

    #[test]
    fn values_render_the_way_platipus_writes_them() {
        assert_eq!(Value::Null.to_string(), "null");
        assert_eq!(Value::int(3).to_string(), "3");
        assert_eq!(
            Value::List(vec![Value::int(1), Value::text("x")]).to_string(),
            "[1, x]"
        );
    }

    #[test]
    fn truthiness_follows_the_language_rules() {
        assert!(!Value::Null.truthy());
        assert!(!Value::text("").truthy());
        assert!(Value::text("x").truthy());
        assert!(!Value::int(0).truthy());
        assert!(Value::int(1).truthy());
    }
}
