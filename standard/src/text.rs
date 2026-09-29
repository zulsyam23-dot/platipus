use platipus_runtime::state::Value;

/// Text composition helpers used by templates and layout code.
///
/// `{}` consumes the next value, while `{0}` refers to one by position.
pub fn format(template: &str, values: &[Value]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut characters = template.chars().peekable();
    let mut cursor = 0usize;
    while let Some(ch) = characters.next() {
        if ch == '}' && characters.peek() == Some(&'}') {
            characters.next();
            out.push('}');
            continue;
        }
        if ch != '{' {
            out.push(ch);
            continue;
        }
        if characters.peek() == Some(&'{') {
            characters.next();
            out.push('{');
            continue;
        }
        let mut placeholder = String::new();
        let mut closed = false;
        for next in characters.by_ref() {
            if next == '}' {
                closed = true;
                break;
            }
            placeholder.push(next);
        }
        if !closed {
            out.push('{');
            out.push_str(&placeholder);
            break;
        }
        let trimmed = placeholder.trim();
        let resolved = if trimmed.is_empty() {
            let value = values.get(cursor);
            cursor += 1;
            value
        } else {
            trimmed.parse::<usize>().ok().and_then(|index| values.get(index))
        };
        match resolved {
            Some(Value::Text(text)) => out.push_str(text),
            Some(other) => out.push_str(&other.to_string()),
            None => {
                out.push('{');
                out.push_str(&placeholder);
                out.push('}');
            }
        }
    }
    out
}

pub fn join(value: &Value, separator: &Value) -> Value {
    let separator = separator.as_text().unwrap_or_default();
    match value {
        Value::List(items) => Value::text(
            items
                .iter()
                .map(|item| match item {
                    Value::Text(text) => text.clone(),
                    other => other.to_string(),
                })
                .collect::<Vec<_>>()
                .join(separator),
        ),
        Value::Text(text) => Value::text(text.clone()),
        _ => Value::text(""),
    }
}

pub fn repeat(value: &Value, times: i64) -> String {
    let text = value.as_text().unwrap_or_default();
    if times <= 0 {
        return String::new();
    }
    text.repeat(times as usize)
}

pub fn pad_start(value: &Value, width: i64) -> String {
    let text = value.as_text().unwrap_or_default();
    let width = width.max(0) as usize;
    let length = text.chars().count();
    if length >= width {
        return text.to_string();
    }
    format!("{}{}", " ".repeat(width - length), text)
}

pub fn pad_end(value: &Value, width: i64) -> String {
    let text = value.as_text().unwrap_or_default();
    let width = width.max(0) as usize;
    let length = text.chars().count();
    if length >= width {
        return text.to_string();
    }
    format!("{}{}", text, " ".repeat(width - length))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholders_are_filled_by_position() {
        assert_eq!(
            format("{} + {} = {}", &[Value::int(1), Value::int(2), Value::int(3)]),
            "1 + 2 = 3"
        );
    }

    #[test]
    fn doubled_braces_are_literal() {
        assert_eq!(format("{{}}", &[]), "{}");
        assert_eq!(format("{{{}}}", &[Value::int(1)]), "{1}");
    }

    #[test]
    fn unknown_placeholders_are_left_alone() {
        assert_eq!(format("hi {name}", &[]), "hi {name}");
        assert_eq!(format("hi {", &[]), "hi {");
    }

    #[test]
    fn joining_renders_every_item() {
        let list = Value::List(vec![Value::text("a"), Value::int(2)]);
        assert_eq!(join(&list, &Value::text("-")), Value::text("a-2"));
    }

    #[test]
    fn padding_reaches_the_requested_width() {
        assert_eq!(pad_start(&Value::text("7"), 3), "  7");
        assert_eq!(pad_end(&Value::text("7"), 3), "7  ");
        assert_eq!(pad_end(&Value::text("long"), 2), "long");
        assert_eq!(pad_end(&Value::text("x"), -1), "x");
    }

    #[test]
    fn repeating_zero_times_is_empty() {
        assert_eq!(repeat(&Value::text("ab"), 0), "");
        assert_eq!(repeat(&Value::text("ab"), 2), "abab");
    }
}
