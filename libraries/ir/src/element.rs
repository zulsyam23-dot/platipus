use platipus_diagnostics::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct IrElement {
    pub name: String,
    pub kind: ElementKind,
    pub properties: Vec<crate::expression::IrProperty>,
    pub bindings: Vec<IrBinding>,
    pub text: Option<crate::expression::IrExpression>,
    pub handlers: Vec<crate::event::IrHandler>,
    pub body: Option<Box<IrElementBody>>,
    pub span: Span,
}

/// A `bind name: expr` link that keeps a DOM field and reactive state in sync
/// in both directions.
#[derive(Debug, Clone, PartialEq)]
pub struct IrBinding {
    pub name: String,
    pub value: crate::expression::IrExpression,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ElementKind {
    Primitive,
    Component,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct IrElementBody {
    pub items: Vec<ElementItem>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementItem {
    Child(crate::element::IrElement),
    Statement(crate::statement::IrStatement),
    Style(crate::style::IrStyleBlock),
    Responsive(crate::element::IrResponsiveBlock),
}

/// A `responsive { }` block attached to an element, mapping breakpoints onto a
/// property override or a named style.
#[derive(Debug, Clone, PartialEq)]
pub struct IrResponsiveBlock {
    pub entries: Vec<IrResponsiveEntry>,
    pub span: Span,
}

/// One entry of a responsive block. Both variants only change how the element
/// is styled; neither replaces the element itself.
#[derive(Debug, Clone, PartialEq)]
pub enum IrResponsiveEntry {
    /// A style property override, lowered to a plain CSS value.
    Property {
        breakpoint: String,
        name: String,
        value: String,
        span: Span,
    },
    /// A top-level `style { }` block whose rules apply at this breakpoint.
    Named {
        breakpoint: String,
        name: String,
        span: Span,
    },
}
