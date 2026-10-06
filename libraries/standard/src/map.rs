use platipus_reactive::state::Value;

/// All keys of a map, in sorted order.
pub fn keys(value: &Value) -> Option<Value> {
    match value {
        Value::Map(entries) => Some(Value::List(
            entries.keys().map(|k| Value::text(k.clone())).collect(),
        )),
        _ => None,
    }
}

/// All values of a map, in key order.
pub fn values(value: &Value) -> Option<Value> {
    match value {
        Value::Map(entries) => Some(Value::List(entries.values().cloned().collect())),
        _ => None,
    }
}

pub fn has(value: &Value, key: &Value) -> Option<Value> {
    match (value, key) {
        (Value::Map(entries), Value::Text(k)) => Some(Value::Bool(entries.contains_key(k))),
        _ => None,
    }
}

pub fn get(value: &Value, key: &Value) -> Option<Value> {
    match (value, key) {
        (Value::Map(entries), Value::Text(k)) => Some(entries.get(k).cloned().unwrap_or(Value::Null)),
        _ => None,
    }
}

/// A new map with the entries of `b` overwriting those of `a`.
pub fn merge(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (Value::Map(x), Value::Map(y)) => {
            let mut out = x.clone();
            for (k, v) in y {
                out.insert(k.clone(), v.clone());
            }
            Some(Value::Map(out))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn map(pairs: &[(&str, Value)]) -> Value {
        let mut entries = BTreeMap::new();
        for (k, v) in pairs {
            entries.insert(k.to_string(), v.clone());
        }
        Value::Map(entries)
    }

    #[test]
    fn keys_values() {
        let m = map(&[("a", Value::int(1)), ("b", Value::int(2))]);
        assert_eq!(
            keys(&m),
            Some(Value::List(vec![Value::text("a"), Value::text("b")]))
        );
        assert_eq!(
            values(&m),
            Some(Value::List(vec![Value::int(1), Value::int(2)]))
        );
    }

    #[test]
    fn has_get_merge() {
        let a = map(&[("x", Value::int(1))]);
        let b = map(&[("x", Value::int(9)), ("y", Value::int(2))]);
        assert_eq!(has(&a, &Value::text("x")), Some(Value::Bool(true)));
        assert_eq!(get(&b, &Value::text("x")), Some(Value::int(9)));
        assert_eq!(
            merge(&a, &b),
            Some(map(&[("x", Value::int(9)), ("y", Value::int(2))]))
        );
    }
}
