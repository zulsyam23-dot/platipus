pub mod checker;
pub mod element;
pub mod events;
pub mod scope;
pub mod types;

pub use checker::SemanticChecker;
pub use element::{ElementDefinition, ElementProperty, ElementRegistry};
pub use scope::{Scope, ScopeKind, ScopeStack, Symbol, SymbolKind};
pub use types::{PrimitiveType, Type, TypeId, TypeRegistry};

pub use crate::ast::StateKind;
pub use crate::diagnostics::{DiagnosticBag, Error, ErrorKind, Note, Span, Warning, WarningKind};
