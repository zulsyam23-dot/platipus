use platipus_reactive::state::Value;

/// Parses a base-10 integer from text; surrounding whitespace is allowed.
pub fn parse_int(value: &Value) -> Option<Value> {
    let text = value.as_text()?;
    text.trim().parse::<i64>().ok().map(Value::int)
}

/// Parses a float from text; surrounding whitespace is allowed.
pub fn parse_float(value: &Value) -> Option<Value> {
    let text = value.as_text()?;
    text.trim().parse::<f64>().ok().map(Value::float)
}

/// The text rendering of any value, mirroring template interpolation.
pub fn to_text(value: &Value) -> Value {
    match value {
        Value::Null => Value::text(""),
        Value::Bool(b) => Value::text(if *b { "true" } else { "false" }),
        Value::Int(n) => Value::text(n.to_string()),
        Value::Float(f) => Value::text(f.to_string()),
        Value::Text(t) => Value::text(t.clone()),
        Value::List(items) => Value::text(
            items
                .iter()
                .map(|v| match to_text(v) {
                    Value::Text(t) => t,
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>()
                .join(", "),
        ),
        Value::Map(_) => Value::text("[map]"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_int_float() {
        assert_eq!(parse_int(&Value::text(" 42 ")), Some(Value::int(42)));
        assert_eq!(parse_int(&Value::text("nope")), None);
        assert_eq!(parse_float(&Value::text("3.5")), Some(Value::float(3.5)));
    }

    #[test]
    fn to_text_renders_scalars() {
        assert_eq!(to_text(&Value::Bool(true)), Value::text("true"));
        assert_eq!(to_text(&Value::int(7)), Value::text("7"));
        assert_eq!(to_text(&Value::Null), Value::text(""));
    }
}
