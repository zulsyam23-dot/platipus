use platipus_diagnostics::{Error, ErrorKind, Span};
use crate::IrModule;

/// A single file produced by a codegen target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub name: &'static str,
    pub contents: String,
}

impl Artifact {
    pub fn new(name: &'static str, contents: impl Into<String>) -> Self {
        Self {
            name,
            contents: contents.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodegenError {
    pub code: &'static str,
    pub message: String,
    pub span: Option<Span>,
}

impl CodegenError {
    pub fn new(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        Self {
            code,
            message: message.into(),
            span: Some(span),
        }
    }

    pub fn into_error(self) -> Error {
        let mut error = Error::new(ErrorKind::Codegen, self.code, self.message);
        if let Some(span) = self.span {
            error = error.with_span(span);
        }
        error
    }
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

pub type CodegenResult<T> = Result<T, CodegenError>;

/// The result of compiling a program's `#[rust]` blocks: export metadata
/// plus the compiled artifact a backend may consume (wasm today). Produced
/// by the compiler's rust stage and handed to backends via `Target::generate`;
/// it is intentionally not part of `IrModule`, which must stay platform-neutral.
#[derive(Debug, Clone, PartialEq)]
pub struct RustBridge {
    pub exports: Vec<platipus_language::ast::rust::RustExport>,
    /// Base64 of the compiled wasm module, present when a wasm build ran.
    pub wasm_b64: Option<String>,
}

/// Translates a verified module into concrete build artifacts.
pub trait Target {
    fn name(&self) -> &'static str;

    fn generate(
        &self,
        module: &IrModule,
        rust: Option<&RustBridge>,
    ) -> CodegenResult<Vec<Artifact>>;
}
