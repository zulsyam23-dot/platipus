use platipus_diagnostics::DiagnosticBag;
use platipus_language::ast::Program;
use platipus_language::element::ElementRegistry;
use crate::scope::{ScopeKind, ScopeStack, Symbol, SymbolKind};
use crate::SemanticChecker;

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
        span: platipus_diagnostics::Span::empty(),
    });
}
