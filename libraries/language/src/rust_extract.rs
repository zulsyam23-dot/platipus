use crate::ast::Program;
use platipus_diagnostics::{DiagnosticBag, Error, ErrorKind, Span};

pub use crate::ast::rust::RustExport;

/// The validated result of turning every `#[rust]` block into one Rust
/// compilation unit.
#[derive(Debug, Clone)]
pub struct RustExtraction {
    /// The combined, `#[export]`-stripped, impl-renamed Rust source.
    pub source: String,
    /// Offset of `source` relative to the `.plt` file (start of first block's
    /// source). Used to map cargo diagnostics; blocks after the first are
    /// separated by newline comments each recorded in `block_offsets`.
    pub source_start: u32,
    pub block_offsets: Vec<u32>,
    pub exports: Vec<RustExport>,
}

/// Collects every `#[rust]` block in the program, strips `#[export]`
/// markers (replacing them with spaces to keep byte offsets), renames the
/// original function bodies, and validates signatures.
pub fn extract_rust(program: &Program) -> Result<Option<RustExtraction>, DiagnosticBag> {
    if program.rust_blocks.is_empty() {
        return Ok(None);
    }
    let mut bag = DiagnosticBag::new();
    let mut combined = String::new();
    let mut block_offsets = Vec::new();
    let mut exports = Vec::new();
    let mut source_start_of_first: Option<u32> = None;

    for block in &program.rust_blocks {
        let mut source = block.source.clone();
        let mut block_exports: Vec<RustExport> = Vec::new();

        if block.is_exported_all() {
            // Every top-level fn is exported.
            for off in find_top_level_fns(&source) {
                if let Some(ExportDraft::Signature(name, params, return_type)) =
                    parse_fn_signature(&source, off)
                {
                    block_exports.push(RustExport {
                        name: name.clone(),
                        params,
                        return_type,
                        impl_name: format!("__plt_impl_{name}"),
                        source_offset: off,
                        plt_offset: block.source_start as usize + off,
                        wasm_name: None,
                    });
                }
            }
        } else {
            // Find `#[export]` markers inside the raw source before stripping.
            let mut search = 0usize;
            while let Some(pos) = source[search..].find("#[export]") {
                let abs = search + pos;
                let marker_end = abs + "#[export]".len();
                let remainder = &source[marker_end..];
                match find_fn_after_ws(remainder) {
                    Some(fn_pos) => {
                        match parse_fn_signature(&source, marker_end + fn_pos) {
                            Some(ExportDraft::Signature(name, params, return_type)) => {
                                block_exports.push(RustExport {
                                    name,
                                    params,
                                    return_type,
                                    impl_name: String::new(),
                                    source_offset: marker_end + fn_pos,
                                    plt_offset: block.source_start as usize + marker_end + fn_pos,
                                    wasm_name: None,
                                });
                            }
                            _ => {
                                bag.error(
                                    Error::new(
                                        ErrorKind::Semantic,
                                        "export-outside-fn",
                                        "`#[export]` must precede a function declaration",
                                    )
                                    .with_span(Span::new(
                                        block.source_start + abs as u32,
                                        block.source_start + abs as u32 + 9,
                                    )),
                                );
                            }
                        }
                    }
                    None => {
                        bag.error(
                            Error::new(
                                ErrorKind::Semantic,
                                "export-outside-fn",
                                "`#[export]` must precede a function declaration",
                            )
                                    .with_span(Span::new(
                                        block.source_start + abs as u32,
                                        block.source_start + abs as u32 + 9,
                                    ))
                                    .with_help("write `#[export] fn name(...) { ... }`"),
                                );
                    }
                }
                search = marker_end;
            }
        }

        // Now that per-fn markers drove export detection, strip every
        // `#[export]` marker from the source (replace with spaces to keep
        // byte offsets stable for diagnostics).
        let mut strip_search = 0usize;
        while let Some(pos) = source[strip_search..].find("#[export]") {
            let abs = strip_search + pos;
            source.replace_range(abs..abs + "#[export]".len(), &" ".repeat("#[export]".len()));
            strip_search = abs + "#[export]".len();
        }

        // Rename the original fns so the extern wrappers can reuse the names.
        let mut out = String::with_capacity(source.len());
        let mut last = 0usize;
        let mut spans: Vec<(usize, usize, String)> = Vec::new();
        for export in &block_exports {
            let name_start = export.source_offset + 3;
            spans.push((name_start, name_start + export.name.len(), export.name.clone()));
        }
        spans.sort_by_key(|(start, _, _)| *start);
        for (start, end, name) in spans {
            out.push_str(&source[last..start]);
            out.push_str(&format!("__plt_impl_{name}"));
            last = end;
        }
        out.push_str(&source[last..]);

        for export in &mut block_exports {
            export.impl_name = format!("__plt_impl_{}", export.name);
        }

        // Type validation.
        for export in &block_exports {
            if let Err(error) = crate::rust_types::check_signature(export) {
                bag.error(
                    Error::new(
                        ErrorKind::Semantic,
                        "unsupported-rust-type",
                        format!("Rust function `{}` uses unsupported type `{}`", export.name, error.0),
                    )
                    .with_span(Span::new(export.plt_offset as u32, export.plt_offset as u32))
                    .with_help("supported types: i64, i32, i16, i8, u64, u32, u16, u8, f64, f32, bool, String"),
                );
            }
        }

        if source_start_of_first.is_none() {
            source_start_of_first = Some(block.source_start);
        }
        block_offsets.push(block.source_start);
        if !combined.is_empty() {
            combined.push('\n');
        }
        combined.push_str(&out);
        exports.extend(block_exports);
    }

    if bag.has_errors() {
        return Err(bag);
    }
    Ok(Some(RustExtraction {
        source: combined,
        source_start: source_start_of_first.unwrap_or(0),
        block_offsets,
        exports,
    }))
}

enum ExportDraft {
    Signature(String, Vec<(String, String)>, Option<String>),
    NotFn,
}

/// Finds `fn` keywords at brace depth 0 (top-level functions).
fn find_top_level_fns(source: &str) -> Vec<usize> {
    let bytes = source.as_bytes();
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'"' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            _ if depth == 0 && bytes[i] == b'f' && source[i..].starts_with("fn ") => {
                out.push(i);
                i += 3;
            }
            _ => {}
        }
        i += 1;
    }
    out
}

/// Parses `fn name(params) -> Ret {`-style text starting at the `fn`
/// keyword. Returns name, params, return type, and the offset after the
/// signature.
fn parse_fn_signature(source: &str, fn_pos: usize) -> Option<ExportDraft> {
    let rest = &source[fn_pos..];
    // Skip "fn "
    if !rest.starts_with("fn ") {
        return Some(ExportDraft::NotFn);
    }
    let name_start = 3;
    let name_end = rest[name_start..]
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .map(|i| name_start + i)?;
    let name = rest[name_start..name_end].to_string();
    let after_name = &rest[name_end..];
    let paren = after_name.find('(')?;
    let open = name_end + paren;
    // Find matching close paren.
    let mut depth = 0i32;
    let mut end_paren = None;
    for (i, ch) in rest[open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    end_paren = Some(open + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let end_paren = end_paren?;
    let params_text = &rest[open + 1..end_paren];
    let mut params = Vec::new();
    for part in split_top_level(params_text, ',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if part == "self" || part.starts_with("&self") || part.starts_with("mut self") {
            return None;
        }
        let (name, ty) = part.split_once(':')?;
        params.push((name.trim().to_string(), ty.trim().to_string()));
    }
    let after_paren = rest[end_paren + 1..].trim_start();
    let return_type = if let Some(stripped) = after_paren.strip_prefix("->") {
        let end = stripped
            .find('{')
            .or_else(|| stripped.find(';'))
            .unwrap_or(stripped.len());
        Some(stripped[..end].trim().to_string())
    } else {
        None
    };
    Some(ExportDraft::Signature(name, params, return_type))
}

fn split_top_level(text: &str, sep: char) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut last = 0;
    for (i, ch) in text.char_indices() {
        match ch {
            '(' | '[' | '<' => depth += 1,
            ')' | ']' | '>' => depth -= 1,
            _ if ch == sep && depth == 0 => {
                out.push(&text[last..i]);
                last = i + 1;
            }
            _ => {}
        }
    }
    out.push(&text[last..]);
    out
}

fn find_fn_after_ws(text: &str) -> Option<usize> {
    let trimmed = text.trim_start();
    if trimmed.starts_with("fn ") {
        Some(text.len() - trimmed.len())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::AppDecl;
    use platipus_diagnostics::Span;

    fn program_with_block(source: &str, attrs: Vec<String>, block_start: u32) -> Program {
        let mut program = Program::new("main.plt");
        program.rust_blocks.push(crate::ast::RustBlock::new(
            attrs,
            source.to_string(),
            Span::new(block_start, block_start + source.len() as u32),
            block_start,
        ));
        program.app = Some(AppDecl {
            name: crate::ast::Identifier::new("Main", Span::empty()),
            inputs: Vec::new(),
            body: Vec::new(),
            span: Span::empty(),
        });
        program
    }

    #[test]
    fn extracts_an_exported_function() {
        let src = "#[export]\nfn add(a: i64, b: i64) -> i64 {\n    a + b\n}\n";
        let program = program_with_block(src, vec!["rust".into()], 0);
        let extracted = extract_rust(&program).unwrap().unwrap();
        assert_eq!(extracted.exports.len(), 1);
        assert_eq!(extracted.exports[0].name, "add");
        assert_eq!(extracted.exports[0].params.len(), 2);
        assert!(!extracted.source.contains("#[export]"));
        assert!(extracted.source.contains("__plt_impl_add"));
    }

    #[test]
    fn exported_all_exports_every_fn() {
        let src = "fn a() -> i64 { 1 }\nfn b(x: i64) -> i64 { x }\n";
        let program = program_with_block(src, vec!["rust".into(), "export".into()], 0);
        let extracted = extract_rust(&program).unwrap().unwrap();
        assert_eq!(extracted.exports.len(), 2);
    }

    #[test]
    fn unsupported_type_errors() {
        let src = "#[export]\nfn foo(x: HashMap<String, i64>) -> i64 { 0 }\n";
        let program = program_with_block(src, vec!["rust".into()], 0);
        let bag = extract_rust(&program).err().unwrap();
        assert!(bag.errors().any(|e| e.message.contains("HashMap")));
    }
}
