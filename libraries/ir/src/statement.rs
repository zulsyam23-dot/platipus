use platipus_diagnostics::Span;
use crate::element::IrElement;

#[derive(Debug, Clone, PartialEq)]
pub struct IrStatement {
    pub kind: StatementKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    Assign {
        target: String,
        operator: String,
        value: String,
    },
    Expression(String),
    If {
        condition: String,
        then_branch: Vec<IrStatement>,
        else_branch: Option<Vec<IrStatement>>,
    },
    For {
        binding: String,
        iterable: String,
        body: Vec<IrStatement>,
    },
    /// `for i in a..b` / `for i in a..=b`: counted, not iterated.
    ForRange {
        binding: String,
        start: String,
        end: String,
        inclusive: bool,
        body: Vec<IrStatement>,
    },
    /// `let name = value`: a plain local, never reactive.
    Let {
        name: String,
        value: String,
    },
    /// `while condition { ... }`.
    While {
        condition: String,
        body: Vec<IrStatement>,
    },
    Return(Option<String>),
    Break,
    Continue,
    Try {
        binding: Option<String>,
        body: Vec<IrStatement>,
        handler: Vec<IrStatement>,
    },
    Emit {
        event: String,
        payload: Option<String>,
    },
    Block(Vec<IrStatement>),
    /// An element built inside a function body; the generated code returns it.
    Render(Box<IrElement>),
    /// A stray `;` — carries no behaviour and may be dropped.
    NoOp,
    /// A declaration written where only a runtime statement is allowed.
    MisplacedDeclaration {
        kind: &'static str,
    },
}

impl StatementKind {
    pub const fn is_noop(&self) -> bool {
        matches!(self, StatementKind::NoOp)
    }
}

impl IrStatement {
    pub fn new(kind: StatementKind, span: Span) -> Self {
        Self { kind, span }
    }
}
