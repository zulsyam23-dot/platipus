pub mod bundle;
pub mod css;
pub mod dom;
pub mod html;
pub mod javascript;
pub mod tests;

use crate::codegen::{Artifact, CodegenResult, Target};
use crate::ir::IrModule;

pub fn generate(module: &IrModule) -> CodegenResult<Vec<Artifact>> {
    let mut artifacts = vec![
        Artifact::new("index.html", html::render(module)),
        Artifact::new("app.css", css::render(module)),
        Artifact::new("app.js", javascript::render(module)),
    ];
    // The shim and the runner only make sense to a test run, so they are not
    // part of what a browser is served.
    artifacts.push(Artifact::new("dom.mjs", dom::render()));
    artifacts.push(Artifact::new("tests.mjs", tests::render(module)));
    artifacts.push(Artifact::new("program.json", bundle::manifest(module)));
    Ok(artifacts)
}

pub struct WebTarget;

impl Target for WebTarget {
    fn name(&self) -> &'static str {
        "web"
    }

    fn generate(&self, module: &IrModule) -> CodegenResult<Vec<Artifact>> {
        generate(module)
    }
}
