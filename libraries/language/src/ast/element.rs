use super::expression::{Expression, Identifier};
use platipus_diagnostics::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyValue {
    pub name: Identifier,
    pub value: Expression,
    pub span: Span,
}

impl PropertyValue {
    pub fn new(name: Identifier, value: Expression, span: Span) -> Self {
        Self { name, value, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingDecl {
    pub property: Identifier,
    pub target: Expression,
    pub span: Span,
}

impl BindingDecl {
    pub fn target_name(&self) -> Option<&Identifier> {
        self.target.as_identifier()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub name: Identifier,
    pub arguments: Vec<PropertyValue>,
    pub text: Option<Expression>,
    pub body: Option<ElementBody>,
    pub span: Span,
}

impl Element {
    pub fn new(name: Identifier, span: Span) -> Self {
        Self {
            name,
            arguments: Vec::new(),
            text: None,
            body: None,
            span,
        }
    }

    pub fn has_body(&self) -> bool {
        self.body.is_some()
    }

    pub fn items(&self) -> &[ElementItem] {
        self.body
            .as_ref()
            .map(|body| body.items.as_slice())
            .unwrap_or(&[])
    }

    pub fn handlers(&self) -> impl Iterator<Item = &super::event::EventHandlerDecl> {
        self.items().iter().filter_map(|item| match item {
            ElementItem::Handler(handler) => Some(handler),
            _ => None,
        })
    }

    pub fn properties(&self) -> Vec<&PropertyValue> {
        self.body
            .as_ref()
            .map(|body| body.properties())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElementBody {
    pub items: Vec<ElementItem>,
    pub span: Span,
}

impl ElementBody {
    pub fn new(items: Vec<ElementItem>, span: Span) -> Self {
        Self { items, span }
    }

    pub fn properties(&self) -> Vec<&PropertyValue> {
        self.items
            .iter()
            .filter_map(|item| match item {
                ElementItem::Property(property) => Some(property),
                _ => None,
            })
            .collect()
    }

    pub fn bindings(&self) -> Vec<&BindingDecl> {
        self.items
            .iter()
            .filter_map(|item| match item {
                ElementItem::Binding(binding) => Some(binding),
                _ => None,
            })
            .collect()
    }

    pub fn children(&self) -> Vec<&Element> {
        self.items
            .iter()
            .filter_map(|item| match item {
                ElementItem::Child(element) => Some(element),
                _ => None,
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementItem {
    Property(PropertyValue),
    Binding(BindingDecl),
    Handler(super::event::EventHandlerDecl),
    Style(super::style::StyleBlock),
    Responsive(super::style::ResponsiveBlock),
    Child(Element),
    Stmt(super::statement::Statement),
}
