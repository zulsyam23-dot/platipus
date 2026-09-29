use std::fmt;

use super::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    Lex,
    Parse,
    Semantic,
    Ir,
    Codegen,
    Cli,
    Io,
}

impl ErrorKind {
    pub const fn stage(self) -> &'static str {
        match self {
            ErrorKind::Lex => "lex",
            ErrorKind::Parse => "parse",
            ErrorKind::Semantic => "semantic",
            ErrorKind::Ir => "ir",
            ErrorKind::Codegen => "codegen",
            ErrorKind::Cli => "cli",
            ErrorKind::Io => "io",
        }
    }

    pub const fn exit_code(self) -> i32 {
        match self {
            ErrorKind::Io | ErrorKind::Cli => 2,
            _ => 1,
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.stage())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub message: String,
    pub span: Option<Span>,
}

impl Note {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }

    pub fn at(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span: Some(span),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub code: &'static str,
    pub message: String,
    pub span: Option<Span>,
    pub notes: Vec<Note>,
    pub help: Option<String>,
}

impl Error {
    pub fn new(kind: ErrorKind, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind,
            code,
            message: message.into(),
            span: None,
            notes: Vec::new(),
            help: None,
        }
    }

    pub fn at(message: impl Into<String>, code: &'static str, span: Span) -> Self {
        Self {
            kind: ErrorKind::Semantic,
            code,
            message: message.into(),
            span: Some(span),
            notes: Vec::new(),
            help: None,
        }
    }

    pub fn with_kind(mut self, kind: ErrorKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_note(mut self, note: Note) -> Self {
        self.notes.push(note);
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn label(&self) -> String {
        format!("{}[{}]: {}", self.kind.stage(), self.code, self.message)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl std::error::Error for Error {}
