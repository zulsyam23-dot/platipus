#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Null,
    /// The attribute names from the leading `#[...]` sequence (always
    /// containing `rust`) plus the byte offset where the raw Rust source
    /// begins; the raw source is the token's lexeme.
    RustBlock(Vec<String>, u32),
}

impl Literal {
    pub fn type_name(&self) -> &'static str {
        match self {
            Literal::Int(_) => "Int",
            Literal::Float(_) => "Float",
            Literal::Str(_) => "String",
            Literal::Bool(_) => "Bool",
            Literal::Null => "Null",
            Literal::RustBlock(..) => "rust",
        }
    }
}

pub fn parse_int_literal(digits: &str) -> Result<i64, String> {
    let cleaned: String = digits.chars().filter(|c| *c != '_').collect();
    if cleaned.is_empty() {
        return Err("integer literal has no digits".into());
    }
    cleaned
        .parse::<i64>()
        .map_err(|_| format!("integer literal `{cleaned}` is out of range for Int"))
}

pub fn parse_float_literal(digits: &str) -> Result<f64, String> {
    let cleaned: String = digits.chars().filter(|c| *c != '_').collect();
    if cleaned.is_empty() {
        return Err("float literal has no digits".into());
    }
    cleaned
        .parse::<f64>()
        .map_err(|_| format!("float literal `{cleaned}` is not a valid Float"))
}

pub fn unescape(raw: &str) -> Result<String, String> {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        let escaped = chars
            .next()
            .ok_or_else(|| "string ends with an incomplete escape".to_string())?;
        match escaped {
            'n' => out.push('\n'),
            't' => out.push('\t'),
            'r' => out.push('\r'),
            '0' => out.push('\0'),
            '\\' => out.push('\\'),
            '"' => out.push('"'),
            '\'' => out.push('\''),
            'u' => {
                let mut digits = String::new();
                let mut saw_brace = false;
                for next in chars.by_ref() {
                    if next == '{' {
                        saw_brace = true;
                        continue;
                    }
                    if next == '}' {
                        break;
                    }
                    digits.push(next);
                }
                if !saw_brace {
                    return Err("unicode escape `\\u` requires braces: \\u{1F600}".into());
                }
                let code = u32::from_str_radix(&digits, 16).map_err(|_| {
                    format!("unicode escape `\\u{{{digits}}}` is not a valid code point")
                })?;
                out.push(
                    char::from_u32(code)
                        .ok_or_else(|| format!("`\\u{{{digits}}}` is not a valid scalar value"))?,
                );
            }
            other => return Err(format!("unknown escape sequence `\\{other}`")),
        }
    }
    Ok(out)
}
