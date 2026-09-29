use super::component::{ComponentDecl, ComponentItem, InputDecl};
use super::element::Element;
use super::expression::{Expression, Identifier};
use super::style::{StyleDefinition, ThemeDecl};
use crate::diagnostics::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct AppDecl {
    pub name: Identifier,
    pub inputs: Vec<InputDecl>,
    pub body: Vec<ComponentItem>,
    pub span: Span,
}

impl AppDecl {
    pub fn root_element(&self) -> Option<&Element> {
        self.body.iter().find_map(|item| match item {
            ComponentItem::Child(element) => Some(element),
            _ => None,
        })
    }

    pub fn emits(&self) -> Vec<&crate::ast::Identifier> {
        crate::ast::component::collect_component_emits(self.name.as_str(), &self.body)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportDecl {
    pub name: Identifier,
    pub path: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApiMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl ApiMethod {
    pub const fn as_str(self) -> &'static str {
        match self {
            ApiMethod::Get => "get",
            ApiMethod::Post => "post",
            ApiMethod::Put => "put",
            ApiMethod::Delete => "delete",
            ApiMethod::Patch => "patch",
        }
    }

    pub const fn http_verb(self) -> &'static str {
        match self {
            ApiMethod::Get => "GET",
            ApiMethod::Post => "POST",
            ApiMethod::Put => "PUT",
            ApiMethod::Delete => "DELETE",
            ApiMethod::Patch => "PATCH",
        }
    }

    pub const fn has_body(self) -> bool {
        matches!(self, ApiMethod::Post | ApiMethod::Put | ApiMethod::Patch)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApiDecl {
    pub name: Identifier,
    pub routes: Vec<ApiRoute>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApiRoute {
    pub method: ApiMethod,
    pub name: Identifier,
    pub path: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestDecl {
    pub name: Identifier,
    pub steps: Vec<TestStep>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestStep {
    Action {
        name: Identifier,
        argument: Option<Expression>,
        span: Span,
    },
    Expect {
        expression: Expression,
        span: Span,
    },
}

impl TestStep {
    pub fn span(&self) -> Span {
        match self {
            TestStep::Action { span, .. } | TestStep::Expect { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub path: String,
    pub app: Option<AppDecl>,
    pub components: Vec<ComponentDecl>,
    pub styles: Vec<StyleDefinition>,
    pub themes: Vec<ThemeDecl>,
    pub apis: Vec<ApiDecl>,
    pub imports: Vec<ImportDecl>,
    pub tests: Vec<TestDecl>,
    pub span: Span,
}

impl Default for Program {
    fn default() -> Self {
        Self::new("")
    }
}

impl Program {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            app: None,
            components: Vec::new(),
            styles: Vec::new(),
            themes: Vec::new(),
            apis: Vec::new(),
            imports: Vec::new(),
            tests: Vec::new(),
            span: Span::empty(),
        }
    }

    pub fn component(&self, name: &str) -> Option<&ComponentDecl> {
        self.components
            .iter()
            .find(|component| component.name.as_str() == name)
    }

    pub fn style(&self, name: &str) -> Option<&StyleDefinition> {
        self.styles.iter().find(|style| style.name.as_str() == name)
    }

    pub fn theme(&self, name: &str) -> Option<&ThemeDecl> {
        self.themes.iter().find(|theme| theme.name.as_str() == name)
    }

    pub fn is_empty(&self) -> bool {
        self.app.is_none()
            && self.components.is_empty()
            && self.styles.is_empty()
            && self.themes.is_empty()
            && self.apis.is_empty()
            && self.imports.is_empty()
            && self.tests.is_empty()
    }
}
