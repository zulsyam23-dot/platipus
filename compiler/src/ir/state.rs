use crate::diagnostics::Span;
use crate::ir::expression::{IrExpression, IrStateKind};

#[derive(Debug, Clone, PartialEq)]
pub struct IrState {
    pub name: String,
    pub kind: IrStateKind,
    pub type_name: Option<String>,
    pub initializer: Option<IrExpression>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrDerived {
    pub name: String,
    pub type_name: Option<String>,
    pub value: String,
    pub dependencies: Vec<String>,
    pub span: Span,
}
