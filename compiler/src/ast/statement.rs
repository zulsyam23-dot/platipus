use super::expression::{AssignOp, Expression, Identifier};
use crate::diagnostics::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

impl Block {
    pub fn new(statements: Vec<Statement>, span: Span) -> Self {
        Self { statements, span }
    }

    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    State(super::state::StateDecl),
    Derived(super::state::DerivedDecl),
    Function(super::function::FunctionDecl),
    Element(super::element::Element),
    Handler(super::event::EventHandlerDecl),
    Emit(super::event::EmitStatement),
    Assignment(Assignment),
    Expression(Expression),
    If(IfStatement),
    For(ForStatement),
    Return(ReturnStatement),
    Break(Span),
    Continue(Span),
    Try(TryStatement),
    Block(Block),
    Import(super::app::ImportDecl),
    Test(super::app::TestDecl),
    Empty(Span),
}

impl Statement {
    pub fn span(&self) -> Span {
        match self {
            Statement::State(decl) => decl.span,
            Statement::Derived(decl) => decl.span,
            Statement::Function(decl) => decl.span,
            Statement::Element(element) => element.span,
            Statement::Handler(handler) => handler.span,
            Statement::Emit(emit) => emit.span,
            Statement::Assignment(assignment) => assignment.span,
            Statement::Expression(expression) => expression.span(),
            Statement::If(statement) => statement.span,
            Statement::For(statement) => statement.span,
            Statement::Return(statement) => statement.span,
            Statement::Break(span) | Statement::Continue(span) | Statement::Empty(span) => *span,
            Statement::Try(statement) => statement.span,
            Statement::Block(block) => block.span,
            Statement::Import(decl) => decl.span,
            Statement::Test(decl) => decl.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub target: Expression,
    pub op: AssignOp,
    pub value: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_branch: Block,
    pub else_branch: Option<Box<ElseBranch>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElseBranch {
    Block(Block),
    If(IfStatement),
}

impl IfStatement {
    pub fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForStatement {
    pub binding: Identifier,
    pub iterable: Expression,
    pub body: Block,
    pub span: Span,
}

impl ForStatement {
    pub fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStatement {
    pub value: Option<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryStatement {
    pub body: Block,
    pub binding: Option<Identifier>,
    pub handler: Block,
    pub span: Span,
}

impl TryStatement {
    pub fn span(&self) -> Span {
        self.span
    }
}
