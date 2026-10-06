pub mod app;
pub mod component;
pub mod element;
pub mod event;
pub mod expression;
pub mod function;
pub mod rust;
pub mod state;
pub mod statement;
pub mod style;

pub use app::{ApiDecl, ApiMethod, ApiRoute, AppDecl, ImportDecl, Program, TestDecl, TestStep};
pub use component::{ComponentDecl, ComponentItem, InputDecl};
pub use element::{BindingDecl, Element, ElementBody, ElementItem, PropertyValue};
pub use event::{EmitStatement, EventCategory, EventHandlerDecl, known, lifecycle};
pub use expression::{
    AssignOp, BinaryOp, Expression, Identifier, LogicalOp, ObjectEntry, PropertyKey, TypeExpr,
    UnaryOp,
};
pub use function::{FunctionDecl, Parameter};
pub use rust::RustBlock;
pub use state::collect_identifiers;
pub use state::{DerivedDecl, StateDecl, StateKind};
pub use statement::{
    Assignment, Block, ElseBranch, ForStatement, IfStatement, ReturnStatement, Statement,
    TryStatement,
};
pub use style::{
    Breakpoint, ResponsiveBlock, ResponsiveEntry, ResponsiveEntryKind, StyleBlock, StyleDefinition,
    StyleEntry, StyleState, StyleStateGroup, ThemeDecl, ThemeToken,
};

pub use platipus_diagnostics::Span;
