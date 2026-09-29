use platipus_runtime::state::Value;

/// Operations over the built-in collection kinds.
pub trait Collection {
    fn length(&self) -> Option<i64>;

    fn contains(&self, needle: &Value) -> bool;

    fn index_of(&self, needle: &Value) -> Option<i64>;
}

impl Collection for Value {
    fn length(&self) -> Option<i64> {
        match self {
            Value::Text(text) => Some(text.chars().count() as i64),
            Value::List(items) => Some(items.len() as i64),
            Value::Map(entries) => Some(entries.len() as i64),
            _ => None,
        }
    }

    fn contains(&self, needle: &Value) -> bool {
        match self {
            Value::Text(text) => match needle {
                Value::Text(needle) => text.contains(needle.as_str()),
                _ => false,
            },
            Value::List(items) => items.iter().any(|item| item == needle),
            Value::Map(entries) => match needle {
                Value::Text(key) => entries.contains_key(key.as_str()),
                _ => false,
            },
            _ => false,
        }
    }

    fn index_of(&self, needle: &Value) -> Option<i64> {
        match self {
            Value::Text(text) => match needle {
                Value::Text(needle) => text.find(needle.as_str()).map(|at| text[..at].chars().count() as i64),
                _ => None,
            },
            Value::List(items) => items.iter().position(|item| item == needle).map(|at| at as i64),
            Value::Map(entries) => match needle {
                Value::Text(key) => entries
                    .keys()
                    .position(|existing| existing == key)
                    .map(|at| at as i64),
                _ => None,
            },
            _ => None,
        }
    }
}

pub fn length(value: &Value) -> Option<i64> {
    value.length()
}

pub fn contains(value: &Value, needle: &Value) -> Option<Value> {
    value.contains(needle).then_some(Value::Bool(true))
}

pub fn index_of(value: &Value, needle: &Value) -> Option<Value> {
    value
        .index_of(needle)
        .map(Value::int)
        .or(Some(Value::int(-1)))
}

pub fn split(value: &Value, separator: &Value) -> Value {
    let (Some(text), Some(separator)) = (value.as_text(), separator.as_text()) else {
        return Value::List(Vec::new());
    };
    if separator.is_empty() {
        return Value::List(text.chars().map(|ch| Value::text(ch.to_string())).collect());
    }
    Value::List(
        text.split(separator)
            .map(|part| Value::text(part.to_string()))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_counts_characters_not_bytes() {
        assert_eq!(length(&Value::text("héllo")), Some(5));
    }

    #[test]
    fn scalars_have_no_length() {
        assert_eq!(length(&Value::int(1)), None);
    }

    #[test]
    fn lists_report_membership_and_position() {
        let list = Value::List(vec![Value::int(1), Value::int(2)]);
        assert_eq!(contains(&list, &Value::int(2)), Some(Value::Bool(true)));
        assert_eq!(index_of(&list, &Value::int(2)), Some(Value::int(1)));
        assert_eq!(index_of(&list, &Value::int(9)), Some(Value::int(-1)));
    }

    #[test]
    fn text_reports_membership_and_position() {
        let text = Value::text("hello");
        assert_eq!(index_of(&text, &Value::text("ll")), Some(Value::int(2)));
        assert!(contains(&text, &Value::text("zz")).is_none());
    }

    #[test]
    fn splitting_uses_the_separator() {
        let parts = split(&Value::text("a,b"), &Value::text(","));
        assert_eq!(parts, Value::List(vec![Value::text("a"), Value::text("b")]));
    }

    #[test]
    fn splitting_an_empty_separator_yields_characters() {
        let parts = split(&Value::text("ab"), &Value::text(""));
        assert_eq!(parts, Value::List(vec![Value::text("a"), Value::text("b")]));
    }
}
