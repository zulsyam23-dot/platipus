use platipus_diagnostics::Span;
use crate::element::IrElement;

#[derive(Debug, Clone, PartialEq)]
pub struct IrComponent {
    pub name: String,
    pub inputs: Vec<IrInput>,
    pub states: Vec<crate::state::IrState>,
    pub derived: Vec<crate::state::IrDerived>,
    pub functions: Vec<IrFunction>,
    pub handlers: Vec<crate::event::IrHandler>,
    pub body: Vec<crate::element::ElementItem>,
    pub emits: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrInput {
    pub name: String,
    pub type_name: Option<String>,
    pub default: Option<String>,
    pub required: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrFunction {
    pub name: String,
    pub is_async: bool,
    pub parameters: Vec<IrInput>,
    pub return_type: Option<String>,
    pub body: Vec<crate::statement::IrStatement>,
    pub span: Span,
}

impl IrComponent {
    pub fn root(&self) -> Option<&IrElement> {
        self.body.iter().find_map(|item| match item {
            crate::element::ElementItem::Child(element) => Some(element),
            _ => None,
        })
    }
}
