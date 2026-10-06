pub mod components;
pub mod elements;
pub mod events;
pub mod expression;
pub mod layout;
pub mod state;
pub mod statement;
pub mod style;
pub mod web;

pub use platipus_ir::backend::{Artifact, CodegenError, CodegenResult, RustBridge, Target};
use platipus_ir::IrModule;

pub struct Web;

impl Target for Web {
    fn name(&self) -> &'static str {
        "web"
    }

    fn generate(
        &self,
        module: &IrModule,
        rust: Option<&RustBridge>,
    ) -> CodegenResult<Vec<Artifact>> {
        web::generate(module, rust)
    }
}
