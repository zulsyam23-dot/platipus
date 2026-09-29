use super::element::{Element, ElementItem};
use super::event::EventHandlerDecl;
use super::expression::{Expression, Identifier, TypeExpr};
use super::function::FunctionDecl;
use super::state::{DerivedDecl, StateDecl};
use super::statement::{ElseBranch, Statement};
use super::style::StyleBlock;
use crate::diagnostics::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct InputDecl {
    pub name: Identifier,
    pub type_annotation: Option<TypeExpr>,
    pub default: Option<Expression>,
    pub span: Span,
}

impl InputDecl {
    pub fn required(&self) -> bool {
        self.default.is_none()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentDecl {
    pub name: Identifier,
    pub inputs: Vec<InputDecl>,
    pub body: Vec<ComponentItem>,
    pub span: Span,
}

impl ComponentDecl {
    pub fn states(&self) -> impl Iterator<Item = &StateDecl> {
        self.body.iter().filter_map(|item| match item {
            ComponentItem::State(decl) => Some(decl),
            _ => None,
        })
    }

    pub fn derived(&self) -> impl Iterator<Item = &DerivedDecl> {
        self.body.iter().filter_map(|item| match item {
            ComponentItem::Derived(decl) => Some(decl),
            _ => None,
        })
    }

    pub fn functions(&self) -> impl Iterator<Item = &FunctionDecl> {
        self.body.iter().filter_map(|item| match item {
            ComponentItem::Function(decl) => Some(decl),
            _ => None,
        })
    }

    pub fn handlers(&self) -> impl Iterator<Item = &EventHandlerDecl> {
        self.body.iter().filter_map(|item| match item {
            ComponentItem::Handler(handler) => Some(handler),
            _ => None,
        })
    }

    pub fn children(&self) -> impl Iterator<Item = &Element> {
        self.body.iter().filter_map(|item| match item {
            ComponentItem::Child(element) => Some(element),
            _ => None,
        })
    }

    pub fn emits(&self) -> Vec<&Identifier> {
        collect_component_emits(self.name.as_str(), &self.body)
    }

    pub fn input(&self, name: &str) -> Option<&InputDecl> {
        self.inputs.iter().find(|input| input.name.as_str() == name)
    }
}

/// Walks a component-like body and returns every custom event it can emit,
/// including the ones nested in functions, handlers, control flow, and child
/// elements.
pub fn collect_component_emits<'a>(_name: &str, body: &'a [ComponentItem]) -> Vec<&'a Identifier> {
    let mut names: Vec<&Identifier> = Vec::new();
    for item in body {
        match item {
            ComponentItem::Stmt(Statement::Emit(emit)) => push_emit(emit, &mut names),
            ComponentItem::Function(decl) => collect_emits(&decl.body.statements, &mut names),
            ComponentItem::Handler(handler) => collect_emits(&handler.body.statements, &mut names),
            ComponentItem::Child(element) => collect_element_emits(element, &mut names),
            _ => {}
        }
    }
    names
}

fn push_emit<'a>(emit: &'a super::event::EmitStatement, names: &mut Vec<&'a Identifier>) {
    if !names.iter().any(|name| name.as_str() == emit.name.as_str()) {
        names.push(&emit.name);
    }
}

fn collect_element_emits<'a>(element: &'a Element, names: &mut Vec<&'a Identifier>) {
    for item in element.items() {
        match item {
            ElementItem::Handler(handler) => collect_emits(&handler.body.statements, names),
            ElementItem::Stmt(Statement::Emit(emit)) => push_emit(emit, names),
            ElementItem::Child(child) => collect_element_emits(child, names),
            _ => {}
        }
    }
}

fn collect_if_else<'a>(branch: &'a ElseBranch, names: &mut Vec<&'a Identifier>) {
    match branch {
        ElseBranch::Block(block) => collect_emits(&block.statements, names),
        ElseBranch::If(nested) => {
            collect_emits(&nested.then_branch.statements, names);
            if let Some(else_branch) = &nested.else_branch {
                collect_if_else(else_branch, names);
            }
        }
    }
}

fn collect_emits<'a>(statements: &'a [Statement], names: &mut Vec<&'a Identifier>) {
    for statement in statements {
        match statement {
            Statement::Emit(emit) => push_emit(emit, names),
            Statement::Function(decl) => collect_emits(&decl.body.statements, names),
            Statement::Handler(handler) => collect_emits(&handler.body.statements, names),
            Statement::Block(block) => collect_emits(&block.statements, names),
            Statement::If(statement) => {
                collect_emits(&statement.then_branch.statements, names);
                if let Some(else_branch) = &statement.else_branch {
                    collect_if_else(else_branch, names);
                }
            }
            Statement::For(statement) => collect_emits(&statement.body.statements, names),
            Statement::Try(statement) => {
                collect_emits(&statement.body.statements, names);
                collect_emits(&statement.handler.statements, names);
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentItem {
    State(StateDecl),
    Derived(DerivedDecl),
    Function(FunctionDecl),
    Handler(EventHandlerDecl),
    Style(StyleBlock),
    Child(Element),
    Stmt(Statement),
    Import(super::app::ImportDecl),
    Test(super::app::TestDecl),
}

impl ComponentItem {
    pub fn span(&self) -> Span {
        match self {
            ComponentItem::State(decl) => decl.span,
            ComponentItem::Derived(decl) => decl.span,
            ComponentItem::Function(decl) => decl.span,
            ComponentItem::Handler(handler) => handler.span,
            ComponentItem::Style(block) => block.span,
            ComponentItem::Child(element) => element.span,
            ComponentItem::Stmt(statement) => statement.span(),
            ComponentItem::Import(decl) => decl.span,
            ComponentItem::Test(decl) => decl.span,
        }
    }
}
