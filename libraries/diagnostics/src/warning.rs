use std::fmt;

use super::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WarningKind {
    Convention,
    UnusedImport,
    UnusedState,
    UnusedFunction,
    UnusedParameter,
    UnreachableCode,
    ImplicitAny,
    UnstableSyntax,
    MissingKey,
    Performance,
    Deprecated,
}

impl WarningKind {
    pub const fn code(self) -> &'static str {
        match self {
            WarningKind::Convention => "convention",
            WarningKind::UnusedImport => "unused-import",
            WarningKind::UnusedState => "unused-state",
            WarningKind::UnusedFunction => "unused-function",
            WarningKind::UnusedParameter => "unused-parameter",
            WarningKind::UnreachableCode => "unreachable-code",
            WarningKind::ImplicitAny => "implicit-any",
            WarningKind::UnstableSyntax => "unstable-syntax",
            WarningKind::MissingKey => "missing-key",
            WarningKind::Performance => "performance",
            WarningKind::Deprecated => "deprecated",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Note,
    Warning,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    pub kind: WarningKind,
    pub severity: Severity,
    pub message: String,
    pub span: Option<Span>,
    pub help: Option<String>,
}

impl Warning {
    pub fn new(kind: WarningKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            severity: Severity::Warning,
            message: message.into(),
            span: None,
            help: None,
        }
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn label(&self) -> String {
        format!("warning[{}]: {}", self.kind.code(), self.message)
    }
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label())
    }
}
