use super::expression::{Expression, Identifier, TypeExpr};
use super::statement::Block;
use platipus_diagnostics::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub is_async: bool,
    pub name: Identifier,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<TypeExpr>,
    pub body: Block,
    pub span: Span,
}

impl FunctionDecl {
    pub fn parameter_names(&self) -> Vec<&str> {
        self.parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: Identifier,
    pub type_annotation: Option<TypeExpr>,
    pub default: Option<Expression>,
    pub span: Span,
}

impl Parameter {
    pub fn required(&self) -> bool {
        self.default.is_none()
    }
}
