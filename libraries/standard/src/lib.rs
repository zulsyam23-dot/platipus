//! The Platipus standard library.
//!
//! The standard library is the set of built-in helpers a program can use
//! without importing anything. Helpers are grouped by the concept they serve,
//! mirroring the way the language keeps a single concept per name.

pub mod collection;
pub mod list;
pub mod map;
pub mod math;
pub mod parse;
pub mod string;
pub mod text;

pub use collection::Collection;
pub use string::{StringOps, trim};
pub use text::{format, join, pad_end, pad_start, repeat};

use platipus_reactive::state::Value;

/// A built-in helper, with the number of arguments it takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Builtin {
    pub name: &'static str,
    pub arity: usize,
}

impl Builtin {
    pub const fn new(name: &'static str, arity: usize) -> Self {
        Self { name, arity }
    }

    pub fn accepts(&self, arguments: usize) -> bool {
        arguments == self.arity
    }
}

/// Every built-in the language exposes.
pub const BUILTINS: &[Builtin] = &[
    Builtin::new("len", 1),
    Builtin::new("isEmpty", 1),
    Builtin::new("contains", 2),
    Builtin::new("indexOf", 2),
    Builtin::new("upper", 1),
    Builtin::new("lower", 1),
    Builtin::new("trim", 1),
    Builtin::new("split", 2),
    Builtin::new("replace", 3),
    Builtin::new("repeat", 2),
    Builtin::new("join", 2),
    Builtin::new("padStart", 2),
    Builtin::new("padEnd", 2),
    Builtin::new("abs", 1),
    Builtin::new("min", 2),
    Builtin::new("max", 2),
    Builtin::new("clamp", 3),
    Builtin::new("floor", 1),
    Builtin::new("ceil", 1),
    Builtin::new("round", 1),
    Builtin::new("sqrt", 1),
    Builtin::new("pow", 2),
    Builtin::new("first", 1),
    Builtin::new("last", 1),
    Builtin::new("take", 2),
    Builtin::new("drop", 2),
    Builtin::new("reverse", 1),
    Builtin::new("unique", 1),
    Builtin::new("keys", 1),
    Builtin::new("values", 1),
    Builtin::new("has", 2),
    Builtin::new("get", 2),
    Builtin::new("merge", 2),
    Builtin::new("parseInt", 1),
    Builtin::new("parseFloat", 1),
    Builtin::new("toText", 1),
];

/// Looks a built-in up by name.
pub fn lookup(name: &str) -> Option<Builtin> {
    BUILTINS
        .iter()
        .find(|builtin| builtin.name == name)
        .copied()
}

/// Runs a built-in against run time values.
pub fn call(name: &str, arguments: &[Value]) -> Option<Value> {
    let value = arguments.first().cloned().unwrap_or(Value::Null);
    match name {
        "len" => collection::length(&value).map(Value::int),
        "isEmpty" => Some(Value::Bool(collection::length(&value).is_none_or(|l| l == 0))),
        "contains" => Some(collection::contains(&value, arguments.get(1)?).unwrap_or(Value::Bool(false))),
        "indexOf" => collection::index_of(&value, arguments.get(1)?),
        "upper" => Some(Value::text(string::upper(&value))),
        "lower" => Some(Value::text(string::lower(&value))),
        "trim" => Some(Value::text(string::trim(&value))),
        "split" => Some(collection::split(&value, arguments.get(1)?)),
        "replace" => {
            let from = arguments.get(1)?.as_text()?.to_string();
            let to = arguments.get(2)?.as_text()?.to_string();
            Some(Value::text(string::replace(
                &value,
                &from,
                &to,
            )))
        }
        "repeat" => {
            let times = arguments.get(1)?.as_int()?;
            Some(Value::text(repeat(&value, times)))
        }
        "join" => Some(join(&value, arguments.get(1)?)),
        "padStart" => Some(Value::text(pad_start(
            &value,
            arguments.get(1)?.as_int()?,
        ))),
        "padEnd" => Some(Value::text(pad_end(
            &value,
            arguments.get(1)?.as_int()?,
        ))),
        "abs" => math::abs(&value),
        "min" => math::min(&value, arguments.get(1)?),
        "max" => math::max(&value, arguments.get(1)?),
        "clamp" => math::clamp(&value, arguments.get(1)?, arguments.get(2)?),
        "floor" => math::floor(&value),
        "ceil" => math::ceil(&value),
        "round" => math::round(&value),
        "sqrt" => math::sqrt(&value),
        "pow" => math::pow(&value, arguments.get(1)?),
        "first" => list::first(&value),
        "last" => list::last(&value),
        "take" => list::take(&value, arguments.get(1)?.as_int()?),
        "drop" => list::drop(&value, arguments.get(1)?.as_int()?),
        "reverse" => list::reverse(&value),
        "unique" => list::unique(&value),
        "keys" => map::keys(&value),
        "values" => map::values(&value),
        "has" => map::has(&value, arguments.get(1)?),
        "get" => map::get(&value, arguments.get(1)?),
        "merge" => map::merge(&value, arguments.get(1)?),
        "parseInt" => parse::parse_int(&value),
        "parseFloat" => parse::parse_float(&value),
        "toText" => Some(parse::to_text(&value)),
        _ => None,
    }
}

/// Every built-in name, in declaration order.
pub fn names() -> Vec<&'static str> {
    BUILTINS.iter().map(|builtin| builtin.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_builtin_is_reachable() {
        for builtin in BUILTINS {
            assert_eq!(lookup(builtin.name), Some(*builtin), "{}", builtin.name);
        }
    }

    #[test]
    fn unknown_builtins_are_rejected() {
        assert_eq!(lookup("frobnicate"), None);
        assert!(call("frobnicate", &[]).is_none());
    }

    #[test]
    fn arity_is_enforced() {
        assert!(lookup("len").unwrap().accepts(1));
        assert!(!lookup("len").unwrap().accepts(0));
        assert!(lookup("replace").unwrap().accepts(3));
    }

    #[test]
    fn length_works_across_value_kinds() {
        assert_eq!(call("len", &[Value::text("abc")]), Some(Value::int(3)));
        assert_eq!(
            call("len", &[Value::List(vec![Value::int(1), Value::int(2)])]),
            Some(Value::int(2))
        );
        assert_eq!(call("len", &[Value::Null]), None);
    }

    #[test]
    fn emptiness_is_reported_for_every_kind() {
        assert_eq!(call("isEmpty", &[Value::text("")]), Some(Value::Bool(true)));
        assert_eq!(call("isEmpty", &[Value::List(vec![])]), Some(Value::Bool(true)));
        assert_eq!(call("isEmpty", &[Value::text("x")]), Some(Value::Bool(false)));
    }

    #[test]
    fn text_transforms_run_in_place() {
        assert_eq!(call("upper", &[Value::text("ab")]), Some(Value::text("AB")));
        assert_eq!(call("lower", &[Value::text("AB")]), Some(Value::text("ab")));
        assert_eq!(call("trim", &[Value::text(" a ")]), Some(Value::text("a")));
    }

    #[test]
    fn replace_swaps_every_occurrence() {
        assert_eq!(
            call(
                "replace",
                &[Value::text("a-b-a"), Value::text("a"), Value::text("z")]
            ),
            Some(Value::text("z-b-z"))
        );
    }

    #[test]
    fn repeat_duplicates_text() {
        assert_eq!(
            call("repeat", &[Value::text("ab"), Value::int(3)]),
            Some(Value::text("ababab"))
        );
    }

    #[test]
    fn names_are_listed_in_order() {
        assert_eq!(names().first(), Some(&"len"));
    }
}
