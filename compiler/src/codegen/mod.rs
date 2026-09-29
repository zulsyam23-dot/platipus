pub mod components;
pub mod elements;
pub mod events;
pub mod expression;
pub mod layout;
pub mod state;
pub mod statement;
pub mod style;
pub mod web;

use crate::diagnostics::{Error, ErrorKind, Span};
use crate::ir::IrModule;

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

/// Translates a verified module into concrete build artifacts.
pub trait Target {
    fn name(&self) -> &'static str;

    fn generate(&self, module: &IrModule) -> CodegenResult<Vec<Artifact>>;
}

pub struct Web;

impl Target for Web {
    fn name(&self) -> &'static str {
        "web"
    }

    fn generate(&self, module: &IrModule) -> CodegenResult<Vec<Artifact>> {
        web::generate(module)
    }
}
