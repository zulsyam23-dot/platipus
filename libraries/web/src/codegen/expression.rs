use std::collections::BTreeMap;

/// Maps Platipus names to their generated JavaScript access paths.
#[derive(Debug, Clone, Default)]
pub struct Scope {
    entries: BTreeMap<String, String>,
}

impl Scope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(&mut self, name: impl Into<String>, access: impl Into<String>) {
        self.entries.insert(name.into(), access.into());
    }

    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    pub fn access(&self, name: &str) -> Option<&str> {
        self.entries.get(name).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }
}

/// Replaces every bare identifier that is bound in `scope` with its access
/// path. Member accesses (`obj.name`) and string contents are left untouched.
/// A JavaScript property access for `name` on `object`. A name that is not a
/// valid identifier has to be written as a computed access, because
/// `events."done"` is not JavaScript.
pub fn member_access(object: &str, name: &str) -> String {
    let mut chars = name.chars();
    let valid = chars.next().is_some_and(is_identifier_start) && chars.all(is_identifier_continue);
    if valid {
        return format!("{object}.{name}");
    }
    format!("{object}[{}]", js_string(name))
}

/// A JS/JSON string literal. Rust's `{:?}` escaping is not JSON-compatible
/// for astral-plane characters (`\u{1F600}` vs surrogate pairs), so name and
/// path strings cross into generated code through here.
pub fn js_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c if (c as u32) < 0x7f => out.push(c),
            c => {
                let code = c as u32;
                if code <= 0xFFFF {
                    out.push_str(&format!("\\u{:04x}", code));
                } else {
                    let hi = 0xD800 + ((code - 0x10000) >> 10);
                    let lo = 0xDC00 + ((code - 0x10000) & 0x3FF);
                    out.push_str(&format!("\\u{:04x}\\u{:04x}", hi, lo));
                }
            }
        }
    }
    out.push('"');
    out
}

pub fn is_identifier_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_' || ch == '$'
}

pub fn is_identifier_continue(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '$'
}

pub fn rewrite(source: &str, scope: &Scope) -> String {
    let mut out = String::with_capacity(source.len());
    let bytes: Vec<char> = source.chars().collect();
    let mut index = 0usize;
    while index < bytes.len() {
        let ch = bytes[index];
        if ch == '"' || ch == '\'' || ch == '`' {
            let quote = ch;
            out.push(ch);
            index += 1;
            while index < bytes.len() {
                let next = bytes[index];
                out.push(next);
                index += 1;
                if next == '\\' && index < bytes.len() {
                    out.push(bytes[index]);
                    index += 1;
                } else if next == quote {
                    break;
                }
            }
            continue;
        }
        if is_identifier_start(ch) {
            let start = index;
            while index < bytes.len() && is_identifier_continue(bytes[index]) {
                index += 1;
            }
            let word: String = bytes[start..index].iter().collect();
            let member = preceded_by_member(&bytes, start);
            let object_key = followed_by_colon(&bytes, index);
            match scope.access(&word) {
                Some(access) if !member && !object_key => out.push_str(access),
                _ => out.push_str(&word),
            }
            continue;
        }
        out.push(ch);
        index += 1;
    }
    out
}

// A word that is followed by `:` (but not `::` or `:=` or `:` of a ternary
// in expression position) is an object-literal key, not a value reference.
fn followed_by_colon(source: &[char], mut index: usize) -> bool {
    while index < source.len() && source[index].is_whitespace() {
        index += 1;
    }
    index < source.len() && source[index] == ':' && source.get(index + 1) != Some(&':') && source.get(index + 1) != Some(&'=')
}

fn preceded_by_member(source: &[char], start: usize) -> bool {
    let mut cursor = start;
    while cursor > 0 {
        cursor -= 1;
        let ch = source[cursor];
        if ch.is_whitespace() {
            continue;
        }
        return ch == '.';
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rewrites_bound_names() {
        let mut scope = Scope::new();
        scope.bind("count", "s.count.value");
        assert_eq!(rewrite("count + 1", &scope), "s.count.value + 1");
    }

    #[test]
    fn keeps_member_accesses() {
        let mut scope = Scope::new();
        scope.bind("value", "s.value.value");
        assert_eq!(rewrite("item.value", &scope), "item.value");
    }

    #[test]
    fn keeps_string_contents() {
        let mut scope = Scope::new();
        scope.bind("count", "s.count.value");
        assert_eq!(rewrite("\"count\"", &scope), "\"count\"");
    }

    #[test]
    fn a_member_access_uses_a_computed_form_when_it_has_to() {
        assert_eq!(member_access("events", "fired"), "events.fired");
        assert_eq!(
            member_access("events", "item-picked"),
            "events[\"item-picked\"]"
        );
        assert_eq!(member_access("events", "2fast"), "events[\"2fast\"]");
    }
}
