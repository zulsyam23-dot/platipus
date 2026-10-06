use std::collections::BTreeMap;

use crate::state::{StateStore, Value};

type Compute<'a> = dyn Fn(&StateStore) -> Value + 'a;

/// A value computed from other state, recomputed when its inputs change.
pub struct Derived<'a> {
    name: String,
    dependencies: Vec<String>,
    compute: Box<Compute<'a>>,
    value: Value,
    computed: bool,
}

impl<'a> Derived<'a> {
    pub fn new(
        name: impl Into<String>,
        dependencies: Vec<String>,
        compute: impl Fn(&StateStore) -> Value + 'a,
    ) -> Self {
        Self {
            name: name.into(),
            dependencies,
            compute: Box::new(compute),
            value: Value::Null,
            computed: false,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn dependencies(&self) -> &[String] {
        &self.dependencies
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn recompute(&mut self, store: &StateStore) -> &Value {
        self.value = (self.compute)(store);
        self.computed = true;
        &self.value
    }

    pub fn is_computed(&self) -> bool {
        self.computed
    }
}

impl std::fmt::Debug for Derived<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Derived")
            .field("name", &self.name)
            .field("dependencies", &self.dependencies)
            .field("value", &self.value)
            .finish()
    }
}

/// The derived values of one component instance.
#[derive(Debug, Default)]
pub struct DerivedSet {
    entries: Vec<Derived<'static>>,
}

impl DerivedSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, derived: Derived<'static>) {
        self.entries.push(derived);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get(&self, name: &str) -> Option<&Derived<'_>> {
        self.entries.iter().find(|entry| entry.name == name)
    }

    /// Recomputes every derived value that was never computed before, plus
    /// every value whose dependencies changed. A derived value that depends
    /// on another derived value is refreshed in the same call: changed names
    /// feed a fixpoint loop so chains settle in one pass. Cycles are bounded
    /// by a maximum number of sweeps (one beyond the entry count) rather than
    /// looping forever.
    pub fn refresh(&mut self, store: &StateStore, changed: &[String]) -> Vec<String> {
        let mut refreshed: Vec<String> = Vec::new();
        let mut frontier = changed.to_vec();
        let max_sweeps = self.entries.len() + 1;
        for _ in 0..max_sweeps {
            let mut next_frontier = Vec::new();
            for index in 0..self.entries.len() {
                let entry = &self.entries[index];
                let touched = entry
                    .dependencies
                    .iter()
                    .any(|dependency| frontier.iter().any(|name| name == dependency));
                if entry.computed && !touched {
                    continue;
                }
                let before = entry.value.clone();
                let was_computed = entry.computed;
                let entry = &mut self.entries[index];
                entry.recompute(store);
                if !was_computed || entry.value != before {
                    if !refreshed.contains(&entry.name) {
                        refreshed.push(entry.name.clone());
                    }
                    next_frontier.push(entry.name.clone());
                }
            }
            if next_frontier.is_empty() {
                break;
            }
            frontier = next_frontier;
        }
        refreshed
    }

    pub fn snapshot(&self) -> BTreeMap<String, Value> {
        self.entries
            .iter()
            .map(|entry| (entry.name.clone(), entry.value.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Scope;

    fn store() -> StateStore {
        let mut store = StateStore::new();
        store.declare(Scope::Instance, "count", Value::int(2));
        store
    }

    #[test]
    fn recomputes_from_state() {
        let mut store = store();
        let mut derived = Derived::new("doubled", vec!["count".into()], |store| {
            Value::int(
                store
                    .get(Scope::Instance, "count")
                    .unwrap()
                    .as_int()
                    .unwrap()
                    * 2,
            )
        });
        assert_eq!(derived.recompute(&store), &Value::int(4));
        store.set(Scope::Instance, "count", Value::int(5));
        assert_eq!(derived.recompute(&store), &Value::int(10));
    }

    #[test]
    fn the_first_refresh_computes_everything() {
        let store = store();
        let mut set = DerivedSet::new();
        set.insert(Derived::new("doubled", vec!["count".into()], |store| {
            Value::int(
                store
                    .get(Scope::Instance, "count")
                    .unwrap()
                    .as_int()
                    .unwrap()
                    * 2,
            )
        }));
        set.insert(Derived::new("label", vec!["other".into()], |_| {
            Value::text("x")
        }));
        let refreshed = set.refresh(&store, &[]);
        assert_eq!(refreshed, vec!["doubled".to_string(), "label".to_string()]);
    }

    #[test]
    fn refresh_only_recomputes_touched_values() {
        let mut store = store();
        let mut set = DerivedSet::new();
        set.insert(Derived::new("doubled", vec!["count".into()], |store| {
            Value::int(
                store
                    .get(Scope::Instance, "count")
                    .unwrap()
                    .as_int()
                    .unwrap()
                    * 2,
            )
        }));
        set.insert(Derived::new("label", vec!["other".into()], |_| {
            Value::text("x")
        }));
        set.refresh(&store, &[]);
        store.set(Scope::Instance, "count", Value::int(5));
        let refreshed = set.refresh(&store, &["count".to_string()]);
        assert_eq!(refreshed, vec!["doubled".to_string()]);
        assert_eq!(set.get("doubled").unwrap().value(), &Value::int(10));
    }

    #[test]
    fn refresh_does_not_recompute_dependencies_that_stopped_changing() {
        let store = store();
        let dependent_runs = std::rc::Rc::new(std::cell::Cell::new(0));
        let runs = std::rc::Rc::clone(&dependent_runs);
        let mut set = DerivedSet::new();
        set.insert(Derived::new("base", vec!["count".into()], |store| {
            Value::int(
                store
                    .get(Scope::Instance, "count")
                    .unwrap()
                    .as_int()
                    .unwrap(),
            )
        }));
        set.insert(Derived::new("dependent", vec!["base".into()], move |_| {
            runs.set(runs.get() + 1);
            Value::int(1)
        }));
        set.refresh(&store, &[]);

        assert_eq!(dependent_runs.get(), 2);
        set.refresh(&store, &[]);
        assert_eq!(dependent_runs.get(), 2);
    }

    #[test]
    fn snapshot_reports_every_value() {
        let store = store();
        let mut set = DerivedSet::new();
        set.insert(Derived::new("doubled", vec!["count".into()], |store| {
            Value::int(
                store
                    .get(Scope::Instance, "count")
                    .unwrap()
                    .as_int()
                    .unwrap()
                    * 2,
            )
        }));
        set.refresh(&store, &[]);
        assert_eq!(set.snapshot().get("doubled"), Some(&Value::int(4)));
    }
}
