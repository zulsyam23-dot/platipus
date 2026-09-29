#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    App,
    Component,
    Input,
    State,
    Derived,
    Function,
    Parameter,
    Variable,
    Element,
    Type,
    Module,
    Api,
    Test,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub span: crate::diagnostics::Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Global,
    Module,
    App,
    Component,
    Function,
    Block,
    Loop,
}

#[derive(Debug)]
pub struct Scope {
    pub kind: ScopeKind,
    symbols: Vec<Symbol>,
}

impl Scope {
    pub fn new(kind: ScopeKind) -> Self {
        Self {
            kind,
            symbols: Vec::new(),
        }
    }

    pub fn define(&mut self, symbol: Symbol) -> bool {
        if self.lookup_local(&symbol.name).is_some() {
            return false;
        }
        self.symbols.push(symbol);
        true
    }

    pub fn lookup_local(&self, name: &str) -> Option<&Symbol> {
        self.symbols.iter().find(|s| s.name == name)
    }

    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }
}

/// Returned when a name is declared twice in the same scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Redefinition;

#[derive(Debug, Default)]
pub struct ScopeStack {
    scopes: Vec<Scope>,
}

impl ScopeStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, kind: ScopeKind) {
        self.scopes.push(Scope::new(kind));
    }

    pub fn pop(&mut self) {
        self.scopes.pop();
    }

    pub fn current(&self) -> Option<&Scope> {
        self.scopes.last()
    }

    pub fn current_mut(&mut self) -> Option<&mut Scope> {
        self.scopes.last_mut()
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.lookup_local(name) {
                return Some(symbol);
            }
        }
        None
    }

    pub fn define(&mut self, symbol: Symbol) -> Result<(), Redefinition> {
        match self.current_mut() {
            Some(scope) => {
                if scope.define(symbol) {
                    Ok(())
                } else {
                    Err(Redefinition)
                }
            }
            None => Err(Redefinition),
        }
    }

    pub fn depth(&self) -> usize {
        self.scopes.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Scope> {
        self.scopes.iter()
    }
}
