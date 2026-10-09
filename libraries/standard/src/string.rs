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

/// The Unicode code point at `index`, counting characters rather than bytes, so
/// it lines up with `len` and with every other index in the language. An index
/// past the end, or a negative one, reports `-1` the way a miss does elsewhere
/// rather than failing, so a caller can walk a string with a loop.
pub fn code_at(value: &Value, index: i64) -> Option<Value> {
    if index < 0 {
        return Some(Value::int(-1));
    }
    let text = value.as_text()?;
    text.chars()
        .nth(index as usize)
        .map(|ch| Value::int(ch as i64))
        .or(Some(Value::int(-1)))
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

    #[test]
    fn a_code_point_is_read_by_character_index() {
        assert_eq!(code_at(&Value::text("abc"), 0), Some(Value::int(97)));
        assert_eq!(code_at(&Value::text("abc"), 2), Some(Value::int(99)));
    }

    #[test]
    fn an_index_past_the_end_reports_a_miss_rather_than_failing() {
        assert_eq!(code_at(&Value::text("abc"), 3), Some(Value::int(-1)));
        assert_eq!(code_at(&Value::text(""), 0), Some(Value::int(-1)));
        assert_eq!(code_at(&Value::text("abc"), -1), Some(Value::int(-1)));
    }

    #[test]
    fn indexing_counts_characters_so_it_lines_up_with_length() {
        // "naïve" is five characters and `ï` is one code point even though it is
        // two bytes, which is what keeps `codeAt` and `len` describing the same
        // string. Index 2 is the diaeresis, not the `i` that follows it.
        assert_eq!(code_at(&Value::text("naïve"), 2), Some(Value::int(239)));
        assert_eq!(code_at(&Value::text("naïve"), 3), Some(Value::int(118)));
        assert_eq!(code_at(&Value::text("naïve"), 4), Some(Value::int(101)));
        assert_eq!(
            Value::text("naïve").as_text().unwrap().chars().count(),
            5
        );
    }

    #[test]
    fn a_non_text_value_has_no_code_points() {
        assert_eq!(code_at(&Value::int(7), 0), None);
    }
}
