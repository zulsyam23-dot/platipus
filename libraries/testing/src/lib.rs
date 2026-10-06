//! Embedded test contract for Platipus.
//!
//! A `test` block inside a `.plt` program is a sequence of steps executed
//! against the live program: actions drive it (`click "+"`), expectations
//! observe it (`expect count == 1`). This crate owns the shared vocabulary
//! every layer reads — the language parser produces steps, semantic analysis
//! validates their actions, the IR carries them, and the backend's runner
//! executes them — so the four layers can never drift apart.

use platipus_diagnostics::Span;

/// The steps a `test` block may name. The runner has to know what each one
/// means, and this is the single list both the checker and the runner read, so a
/// step the runner cannot perform is rejected by the build.
pub const TEST_ACTIONS: &[&str] = &["click"];

/// One step in an embedded test program.
#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    Action {
        name: String,
        argument: Option<String>,
        span: Span,
    },
    Expect {
        expression: String,
        span: Span,
    },
}

impl Step {
    pub fn span(&self) -> Span {
        match self {
            Step::Action { span, .. } | Step::Expect { span, .. } => *span,
        }
    }
}

/// A named embedded test.
#[derive(Debug, Clone, PartialEq)]
pub struct Test {
    pub name: String,
    pub steps: Vec<Step>,
    pub span: Span,
}

/// Whether `name` is a test action the runner knows how to perform.
pub fn is_known_action(name: &str) -> bool {
    TEST_ACTIONS.contains(&name)
}
