use platipus_diagnostics::Span;

/// A `#[rust]` block: raw Rust source kept verbatim inside a Platipus file.
#[derive(Debug, Clone, PartialEq)]
pub struct RustBlock {
    /// Attribute names from the leading `#[...]` sequence, e.g. `["rust"]`
    /// or `["rust", "export"]`.
    pub attributes: Vec<String>,
    /// The raw Rust source, preserved verbatim. `#[export]` markers may
    /// still be present inside; the extractor strips them.
    pub source: String,
    /// Span covering the whole construct, from the first `#` through the
    /// end of the captured Rust source.
    pub span: Span,
    /// Byte offset where the raw Rust source begins inside the `.plt` file.
    pub source_start: u32,
}

impl RustBlock {
    pub fn new(attributes: Vec<String>, source: String, span: Span, source_start: u32) -> Self {
        Self {
            attributes,
            source,
            span,
            source_start,
        }
    }

    pub fn is_exported_all(&self) -> bool {
        self.attributes.iter().any(|name| name == "export")
    }
}

/// One function exported from a Rust block to Platipus.
#[derive(Debug, Clone, PartialEq)]
pub struct RustExport {
    /// Name as visible in Platipus.
    pub name: String,
    /// Parameters as `(name, rust_type)` pairs.
    pub params: Vec<(String, String)>,
    /// Return type, `None` for `()`.
    pub return_type: Option<String>,
    /// The original body has been renamed to this in the compiled source so
    /// the extern wrapper can share the name.
    pub impl_name: String,
    /// Byte offset of the fn inside the stripped block source.
    pub source_offset: usize,
    /// Byte offset inside the `.plt` file.
    pub plt_offset: usize,
    /// Set when the export was found in the generated wasm.
    pub wasm_name: Option<String>,
}

