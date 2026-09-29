use crate::diagnostics::Span;
use crate::ir::component::IrComponent;

#[derive(Debug, Clone, PartialEq)]
pub struct IrModule {
    pub name: String,
    pub components: Vec<IrComponent>,
    pub styles: Vec<crate::ir::style::IrStyleBlock>,
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

/// `test Name { click "+" expect count == 1 }`
#[derive(Debug, Clone, PartialEq)]
pub struct IrTest {
    pub name: String,
    pub steps: Vec<IrTestStep>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrTestStep {
    Action {
        name: String,
        argument: Option<String>,
        span: Span,
    },
    Expect {
        expression: String,
        span: Span,
    },
}

impl IrTestStep {
    pub fn span(&self) -> Span {
        match self {
            IrTestStep::Action { span, .. } | IrTestStep::Expect { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrApi {
    pub name: String,
    pub routes: Vec<IrRoute>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrRoute {
    pub method: crate::ast::ApiMethod,
    pub name: String,
    pub path: String,
    pub span: Span,
}
