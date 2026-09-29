use crate::diagnostics::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct IrHandler {
    pub event: String,
    pub category: crate::ir::expression::IrEventCategory,
    pub is_custom: bool,
    pub body: Vec<crate::ir::statement::IrStatement>,
    pub span: Span,
}
