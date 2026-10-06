use platipus_diagnostics::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct IrHandler {
    pub event: String,
    pub category: crate::expression::IrEventCategory,
    pub is_custom: bool,
    pub body: Vec<crate::statement::IrStatement>,
    pub span: Span,
}
