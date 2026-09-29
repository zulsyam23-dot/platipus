pub mod error;
pub mod span;
pub mod warning;

use std::fmt;

pub use error::{Error, ErrorKind, Note};
pub use span::{SourceFile, Span};
pub use warning::{Severity, Warning, WarningKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Diagnostic {
    Error(Error),
    Warning(Warning),
}

impl Diagnostic {
    pub fn severity(&self) -> Severity {
        match self {
            Diagnostic::Error(_) => Severity::Deny,
            Diagnostic::Warning(warning) => warning.severity,
        }
    }
}

impl From<Error> for Diagnostic {
    fn from(error: Error) -> Self {
        Diagnostic::Error(error)
    }
}

impl From<Warning> for Diagnostic {
    fn from(warning: Warning) -> Self {
        Diagnostic::Warning(warning)
    }
}

#[derive(Debug, Default, Clone)]
pub struct DiagnosticBag {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticBag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, diagnostic: impl Into<Diagnostic>) {
        self.diagnostics.push(diagnostic.into());
    }

    pub fn error(&mut self, error: Error) {
        self.diagnostics.push(Diagnostic::Error(error));
    }

    pub fn warning(&mut self, warning: Warning) {
        self.diagnostics.push(Diagnostic::Warning(warning));
    }

    pub fn extend(&mut self, other: DiagnosticBag) {
        self.diagnostics.extend(other.diagnostics);
    }

    pub fn errors(&self) -> impl Iterator<Item = &Error> {
        self.diagnostics.iter().filter_map(|d| match d {
            Diagnostic::Error(error) => Some(error),
            Diagnostic::Warning(_) => None,
        })
    }

    pub fn warnings(&self) -> impl Iterator<Item = &Warning> {
        self.diagnostics.iter().filter_map(|d| match d {
            Diagnostic::Warning(warning) => Some(warning),
            Diagnostic::Error(_) => None,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }

    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn has_errors(&self) -> bool {
        self.errors().next().is_some()
    }

    pub fn error_count(&self) -> usize {
        self.errors().count()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings().count()
    }

    pub fn into_vec(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    pub fn sorted(&self) -> Vec<Diagnostic> {
        let mut sorted = self.diagnostics.clone();
        sorted.sort_by_key(|d| match d {
            Diagnostic::Error(error) => error.span.map(|s| s.start).unwrap_or(u32::MAX),
            Diagnostic::Warning(warning) => warning.span.map(|s| s.start).unwrap_or(u32::MAX),
        });
        sorted
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError {
    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl CompileError {
    pub fn from_bag(bag: DiagnosticBag) -> Self {
        Self {
            errors: bag.errors().cloned().collect(),
            warnings: bag.warnings().cloned().collect(),
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, error) in self.errors.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }
            write!(f, "{error}")?;
            if let Some(help) = &error.help {
                write!(f, "\n  help: {help}")?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for CompileError {}
