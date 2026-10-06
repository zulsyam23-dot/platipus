pub use platipus_diagnostics as diagnostics;
pub use platipus_ir as ir;
pub use platipus_language::{ast, lexer, parser};
pub use platipus_semantic as semantic;
/// The web backend, re-exported so the compiler crate facade stays stable.
pub use platipus_web::codegen;

pub mod loader;
pub mod pipeline;
pub mod rust;
