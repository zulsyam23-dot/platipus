use platipus_diagnostics::Span;
use crate::component::IrComponent;

pub use platipus_testing::{Step as IrTestStep, Test as IrTest};

#[derive(Debug, Clone, PartialEq)]
pub struct IrModule {
    pub name: String,
    pub components: Vec<IrComponent>,
    pub styles: Vec<crate::style::IrStyleBlock>,
    pub themes: Vec<IrTheme>,
    pub apis: Vec<IrApi>,
    pub imports: Vec<IrImport>,
    pub tests: Vec<IrTest>,
    pub span: Span,
}

/// `import Name from "./path"` — resolved and inlined by the module loader.
#[derive(Debug, Clone, PartialEq)]
pub struct IrImport {
    pub name: String,
    pub path: String,
    pub span: Span,
}

/// `theme Name { color.primary: "#..." }`
#[derive(Debug, Clone, PartialEq)]
pub struct IrTheme {
    pub name: String,
    pub tokens: Vec<IrThemeToken>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrThemeToken {
    pub path: String,
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrApi {
    pub name: String,
    pub routes: Vec<IrRoute>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrRoute {
    pub method: platipus_language::ast::ApiMethod,
    pub name: String,
    pub path: String,
    pub span: Span,
}
