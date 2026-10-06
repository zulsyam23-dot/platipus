use platipus_reactive::state::Value;

/// Case and search operations over text values.
pub struct StringOps;

impl StringOps {
    pub fn upper(value: &Value) -> String {
        upper(value)
    }

    pub fn lower(value: &Value) -> String {
        lower(value)
    }

    pub fn trim(value: &Value) -> String {
        trim(value)
    }

    pub fn replace(value: &Value, from: &str, to: &str) -> String {
        replace(value, from, to)
    }
}

fn text_of(value: &Value) -> String {
    value.as_text().unwrap_or_default().to_string()
}

pub fn upper(value: &Value) -> String {
    text_of(value).to_uppercase()
}

pub fn lower(value: &Value) -> String {
    text_of(value).to_lowercase()
}

pub fn trim(value: &Value) -> String {
    text_of(value).trim().to_string()
}

pub fn replace(value: &Value, from: &str, to: &str) -> String {
    if from.is_empty() {
        return text_of(value);
    }
    text_of(value).replace(from, to)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_changes_follow_locale_free_rules() {
        assert_eq!(upper(&Value::text("straße")), "STRASSE");
        assert_eq!(lower(&Value::text("ABC")), "abc");
    }

    #[test]
    fn non_text_values_render_as_empty() {
        assert_eq!(upper(&Value::int(1)), "");
    }

    #[test]
    fn replacing_an_empty_needle_changes_nothing() {
        assert_eq!(replace(&Value::text("ab"), "", "x"), "ab");
    }

    #[test]
    fn the_struct_exposes_the_same_operations() {
        assert_eq!(StringOps::upper(&Value::text("a")), "A");
    }
}
