use super::expression::{Expression, Identifier, TypeExpr};
use crate::diagnostics::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateKind {
    Local,
    Shared,
    Global,
    Persistent,
    Derived,
}

impl StateKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            StateKind::Local => "state",
            StateKind::Shared => "shared state",
            StateKind::Global => "global state",
            StateKind::Persistent => "persistent state",
            StateKind::Derived => "derived",
        }
    }

    pub const fn is_persistent(self) -> bool {
        matches!(self, StateKind::Persistent)
    }

    pub const fn is_computed(self) -> bool {
        matches!(self, StateKind::Derived)
    }

    pub const fn requires_initializer(self) -> bool {
        matches!(self, StateKind::Derived)
    }

    pub const fn allows_declaration_only(self) -> bool {
        matches!(
            self,
            StateKind::Shared | StateKind::Global | StateKind::Persistent
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateDecl {
    pub kind: StateKind,
    pub name: Identifier,
    pub type_annotation: Option<TypeExpr>,
    pub initializer: Option<Expression>,
    pub span: Span,
}

impl StateDecl {
    pub fn new(kind: StateKind, name: Identifier, span: Span) -> Self {
        Self {
            kind,
            name,
            type_annotation: None,
            initializer: None,
            span,
        }
    }

    pub fn is_mutable(&self) -> bool {
        !self.kind.is_computed()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DerivedDecl {
    pub name: Identifier,
    pub type_annotation: Option<TypeExpr>,
    pub value: Expression,
    pub span: Span,
}

impl DerivedDecl {
    pub fn dependencies(&self) -> Vec<Identifier> {
        let mut found = Vec::new();
        collect_identifiers(&self.value, &mut found);
        found
    }
}

pub(crate) fn collect_identifiers(expression: &Expression, out: &mut Vec<Identifier>) {
    use Expression as E;
    match expression {
        E::Identifier(identifier) => {
            if !out.iter().any(|existing| existing.name == identifier.name) {
                out.push(identifier.clone());
            }
        }
        E::ArrayLiteral(items, _) => {
            for item in items {
                collect_identifiers(item, out);
            }
        }
        E::ObjectLiteral(entries, _) => {
            for entry in entries {
                collect_identifiers(&entry.value, out);
            }
        }
        E::Unary { operand, .. } => collect_identifiers(operand, out),
        E::Binary { left, right, .. } | E::Logical { left, right, .. } => {
            collect_identifiers(left, out);
            collect_identifiers(right, out);
        }
        E::Assign { target, value, .. } => {
            collect_identifiers(target, out);
            collect_identifiers(value, out);
        }
        E::Call {
            callee, arguments, ..
        } => {
            collect_identifiers(callee, out);
            for argument in arguments {
                collect_identifiers(argument, out);
            }
        }
        E::Member { object, .. } => collect_identifiers(object, out),
        E::Index { object, index, .. } => {
            collect_identifiers(object, out);
            collect_identifiers(index, out);
        }
        E::Await { operand, .. } => collect_identifiers(operand, out),
        _ => {}
    }
}
