//! Raw capture of the Rust source that follows `#[rust]`.
//!
//! The Platipus lexer must not re-tokenize Rust; it only needs to find the
//! end of the Rust block. The scanner here tracks brace depth while
//! understanding Rust's string literals, char literals, lifetimes, raw
//! strings, and line/block comments, so a `}` inside a string or comment is
//! never mistaken for the end of the block.

/// Rust item-opening keywords. The block keeps consuming source at brace
/// depth zero while the next item starts with one of these (or `#`), because
/// a `#[rust]` block may hold several items. Platipus top-level keywords
/// (`app`, `component`, `import`, `style`, `theme`, `api`, `test`) are
/// deliberately absent: seeing one ends the Rust block.
const RUST_STARTERS: &[&str] = &[
    "fn", "pub", "use", "mod", "struct", "impl", "trait", "enum", "const", "static", "type",
    "extern", "async", "unsafe", "let", "where", "macro", "move", "for", "while", "if", "match",
    "return",
];

/// Extracts the Rust source beginning at `start` (just past the last leading
/// attribute). `span_start` is the byte offset of the first `#` of the
/// attribute sequence, used for diagnostics.
///
/// Returns `(source, end_offset)` where `source` is the raw text (trimmed of
/// any trailing blank boundary) and `end_offset` is where scanning stopped.
pub fn extract_rust_source(source: &str, start: usize) -> Result<(String, usize), String> {
    let bytes = source.as_bytes();
    let mut i = start;
    let mut depth: i32 = 0;
    let mut boundary = false;
    // Offset just past the last `;` or `}` that closed an item at depth 0.
    let mut boundary_end = start;

    while i < bytes.len() {
        // At a fresh item boundary at depth 0, decide whether the next item
        // belongs to the Rust block or to Platipus.
        if depth == 0 && boundary {
            match peek_item_start(source, i) {
                Some((true, probe)) => {
                    // Continue: the next item is Rust. Skip it raw; reset the
                    // boundary so it is recomputed at the end of this item.
                    i = probe;
                    boundary = false;
                    continue;
                }
                Some((false, _probe)) => {
                    return Ok((source[start..boundary_end].to_string(), boundary_end));
                }
                None => {
                    // Nothing but comments/whitespace/EOF ahead.
                    return Ok((source[start..boundary_end].to_string(), boundary_end));
                }
            }
        }

        let byte = bytes[i];
        match byte {
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                let open = i;
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                if i + 1 >= bytes.len() {
                    return Err(format!("unterminated block comment in Rust block at {open}"));
                }
                i += 2;
            }
            b'r' if bytes.get(i + 1) == Some(&b'"') => {
                // r"..." raw string
                i += 2;
                while i < bytes.len() && bytes[i] != b'"' {
                    i += 1;
                }
                if i >= bytes.len() {
                    return Err("unterminated raw string in Rust block".into());
                }
                i += 1;
            }
            b'r' if bytes.get(i + 1) == Some(&b'#') => {
                // r#"..."#, r##"..."##, ...
                let mut hashes = 0;
                let mut j = i + 1;
                while j < bytes.len() && bytes[j] == b'#' {
                    hashes += 1;
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b'"' {
                    i = j + 1;
                    let mut closed = false;
                    while i < bytes.len() {
                        if bytes[i] == b'"' {
                            let mut ok = true;
                            for k in 0..hashes {
                                if i + 1 + k < bytes.len() && bytes[i + 1 + k] == b'#' {
                                    continue;
                                }
                                ok = false;
                                break;
                            }
                            if ok {
                                i += 1 + hashes;
                                closed = true;
                                break;
                            }
                        }
                        i += 1;
                    }
                    if !closed {
                        return Err("unterminated raw string in Rust block".into());
                    }
                } else {
                    i += 1;
                }
            }
            b'"' => {
                i += 1;
                while i < bytes.len() {
                    match bytes[i] {
                        b'\\' => i += 2,
                        b'"' => {
                            i += 1;
                            break;
                        }
                        b'\n' => {
                            return Err("unterminated string literal in Rust block".into());
                        }
                        _ => i += 1,
                    }
                }
                if i > bytes.len() {
                    return Err("unterminated string literal in Rust block".into());
                }
            }
            b'\'' => {
                // Either a char literal ('a', '\n', '\\'') or a lifetime
                // ('a). Only treat it as a char literal when the shape is
                // exactly 'x' or '\..'.
                if is_char_literal(source, i) {
                    i = consume_char_literal(source, i);
                } else {
                    i += 1;
                }
            }
            b'{' => {
                depth += 1;
                boundary = false;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                i += 1;
                if depth == 0 {
                    boundary = true;
                    boundary_end = i;
                } else if depth < 0 {
                    return Err("unbalanced `}` in Rust block".into());
                }
            }
            b';' if depth == 0 => {
                boundary = true;
                boundary_end = i + 1;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    if depth != 0 {
        return Err("unterminated Rust block: braces are not balanced".into());
    }
    Ok((source[start..boundary_end].to_string(), boundary_end))
}

/// Looks past whitespace and comments; decides whether the upcoming item is
/// Rust (true) or not (false), and returns the offset of its first token.
fn peek_item_start(source: &str, from: usize) -> Option<(bool, usize)> {
    let bytes = source.as_bytes();
    let mut i = from;
    loop {
        match bytes.get(i) {
            None => return None,
            Some(b' ') | Some(b'\t') | Some(b'\r') | Some(b'\n') => i += 1,
            Some(b'/') if bytes.get(i + 1) == Some(&b'/') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            Some(b'/') if bytes.get(i + 1) == Some(&b'*') => {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i += 2;
            }
            Some(b'#') => return Some((true, i)),
            Some(&byte) if byte.is_ascii_alphabetic() || byte == b'_' => {
                let mut end = i;
                while end < bytes.len()
                    && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_')
                {
                    end += 1;
                }
                let word = &source[i..end];
                return Some((RUST_STARTERS.contains(&word), i));
            }
            Some(_) => return Some((false, i)),
        }
    }
}

/// `'a'` or `'\n'`? then it is a char literal; `'a` (lifetime) is not.
fn is_char_literal(source: &str, i: usize) -> bool {
    let bytes = source.as_bytes();
    match bytes.get(i + 1) {
        Some(b'\\') => matches!(bytes.get(i + 3), Some(b'\'')),
        Some(c) if *c != b'\'' => matches!(bytes.get(i + 2), Some(b'\'')),
        _ => false,
    }
}

fn consume_char_literal(source: &str, i: usize) -> usize {
    let bytes = source.as_bytes();
    let mut j = i + 1;
    if bytes.get(j) == Some(&b'\\') {
        j += 1;
    }
    j += 1;
    if bytes.get(j) == Some(&b'\'') {
        j += 1;
    }
    j
}

#[cfg(test)]
mod tests {
    use super::extract_rust_source;

    fn extract(input: &str) -> String {
        extract_rust_source(input, 0).unwrap().0
    }

    #[test]
    fn captures_a_simple_function() {
        assert_eq!(
            extract("fn add(a: i64, b: i64) -> i64 {\n    a + b\n}\n"),
            "fn add(a: i64, b: i64) -> i64 {\n    a + b\n}"
        );
    }

    #[test]
    fn captures_nested_braces() {
        let src = "fn foo() {\n    if true {\n        println!(\"hi\");\n    }\n}\n";
        assert_eq!(extract(src).trim_end(), src.trim_end());
    }

    #[test]
    fn a_brace_in_a_string_does_not_end_the_block() {
        let src = "fn foo() {\n    println!(\"}\");\n}\n";
        assert_eq!(extract(src).trim_end(), src.trim_end());
    }

    #[test]
    fn a_brace_in_a_raw_string_does_not_end_the_block() {
        let src = "fn foo() {\n    let x = r#\"}\"#;\n}\n";
        assert_eq!(extract(src).trim_end(), src.trim_end());
    }

    #[test]
    fn a_brace_in_a_comment_does_not_end_the_block() {
        let src = "fn foo() {\n    // }\n    /* } */\n}\n";
        assert_eq!(extract(src).trim_end(), src.trim_end());
    }

    #[test]
    fn stops_at_the_next_platipus_keyword() {
        let src = "fn foo() {}\n\napp Main {\n}\n";
        assert_eq!(extract(src), "fn foo() {}");
    }

    #[test]
    fn captures_multiple_items() {
        let src = "use std::io;\n\nfn foo() {}\nfn bar() {}\n";
        assert!(extract(src).contains("fn bar() {}"));
    }

    #[test]
    fn unbalanced_braces_are_an_error() {
        assert!(extract_rust_source("fn foo( {", 0).is_err());
    }

    #[test]
    fn a_char_literal_with_brace_is_ignored() {
        let src = "fn foo() {\n    let c = '}';\n}\n";
        assert_eq!(extract(src).trim_end(), src.trim_end());
    }
}

