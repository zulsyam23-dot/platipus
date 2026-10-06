pub mod checker;


pub mod scope;
pub mod types;

pub use checker::SemanticChecker;
pub use platipus_language::element;
pub use platipus_language::element::{ElementDefinition, ElementProperty, ElementRegistry};
pub use scope::{Scope, ScopeKind, ScopeStack, Symbol, SymbolKind};
pub use types::{PrimitiveType, Type, TypeId, TypeRegistry};

pub use platipus_language::ast::StateKind;
pub use platipus_diagnostics::{DiagnosticBag, Error, ErrorKind, Note, Span, Warning, WarningKind};
