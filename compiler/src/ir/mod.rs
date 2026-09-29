pub mod component;
pub mod element;
pub mod event;
pub mod expression;
pub mod lower;
pub mod module;
pub mod state;
pub mod statement;
pub mod style;
pub mod verifier;

pub use component::{IrComponent, IrFunction, IrInput};
pub use element::{
    ElementItem, ElementKind, IrBinding, IrElement, IrElementBody, IrResponsiveBlock,
    IrResponsiveEntry,
};
pub use event::IrHandler;
pub use expression::{IrEventCategory, IrExpression, IrProperty, IrStateKind, IrStyleState};
pub use lower::{Lowering, lower_expression, lower_program};
pub use module::{IrApi, IrImport, IrModule, IrRoute, IrTest, IrTestStep, IrTheme, IrThemeToken};
pub use state::{IrDerived, IrState};
pub use statement::{IrStatement, StatementKind};
pub use style::{IrStyleBlock, IrStyleEntry};
pub use verifier::{IrError, verify};

pub use crate::ir::module::IrModule as IrProgram;
