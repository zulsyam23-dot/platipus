use crate::diagnostics::DiagnosticBag;
use crate::ast::Program;
use crate::semantic::element::ElementRegistry;
use crate::semantic::scope::{ScopeKind, ScopeStack, Symbol, SymbolKind};
use crate::semantic::SemanticChecker;

pub struct NameResolver {
    pub scopes: ScopeStack,
    pub elements: ElementRegistry,
    pub diagnostics: DiagnosticBag,
}

impl NameResolver {
    pub fn new() -> Self {
        let mut scopes = ScopeStack::new();
        scopes.push(ScopeKind::Global);
        Self {
            scopes,
            elements: ElementRegistry::new(),
            diagnostics: DiagnosticBag::new(),
        }
    }

    pub fn resolve(program: &Program) -> DiagnosticBag {
        let mut checker = SemanticChecker::new();
        checker.check(program)
    }
}

impl Default for NameResolver {
    fn default() -> Self {
        Self::new()
    }
}

pub fn declare_component(scopes: &mut ScopeStack, name: &str) {
    scopes.define(Symbol {
        name: name.to_string(),
        kind: SymbolKind::Component,
        span: crate::diagnostics::Span::empty(),
    });
}
