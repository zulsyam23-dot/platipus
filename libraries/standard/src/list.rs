use platipus_reactive::state::Value;

/// The first element of a list, or `None` for empty/other kinds.
pub fn first(value: &Value) -> Option<Value> {
    match value {
        Value::List(items) => items.first().cloned(),
        _ => None,
    }
}

pub fn last(value: &Value) -> Option<Value> {
    match value {
        Value::List(items) => items.last().cloned(),
        _ => None,
    }
}

pub fn take(value: &Value, count: i64) -> Option<Value> {
    match value {
        Value::List(items) => Some(Value::List(
            items.iter().take(count.max(0) as usize).cloned().collect(),
        )),
        _ => None,
    }
}

pub fn drop(value: &Value, count: i64) -> Option<Value> {
    match value {
        Value::List(items) => Some(Value::List(
            items.iter().skip(count.max(0) as usize).cloned().collect(),
        )),
        _ => None,
    }
}

pub fn reverse(value: &Value) -> Option<Value> {
    match value {
        Value::List(items) => {
            let mut reversed = items.clone();
            reversed.reverse();
            Some(Value::List(reversed))
        }
        _ => None,
    }
}

/// A list with duplicate entries removed, first occurrence wins.
pub fn unique(value: &Value) -> Option<Value> {
    match value {
        Value::List(items) => {
            let mut out = Vec::new();
            for item in items {
                if !out.contains(item) {
                    out.push(item.clone());
                }
            }
            Some(Value::List(out))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_last() {
        let list = Value::List(vec![Value::int(1), Value::int(2), Value::int(3)]);
        assert_eq!(first(&list), Some(Value::int(1)));
        assert_eq!(last(&list), Some(Value::int(3)));
    }

    #[test]
    fn take_drop() {
        let list = Value::List(vec![Value::int(1), Value::int(2), Value::int(3)]);
        assert_eq!(
            take(&list, 2),
            Some(Value::List(vec![Value::int(1), Value::int(2)]))
        );
        assert_eq!(
            drop(&list, 1),
            Some(Value::List(vec![Value::int(2), Value::int(3)]))
        );
    }

    #[test]
    fn unique_preserves_order() {
        let list = Value::List(vec![Value::int(2), Value::int(1), Value::int(2), Value::int(1)]);
        assert_eq!(unique(&list), Some(Value::List(vec![Value::int(2), Value::int(1)])));
    }
}
