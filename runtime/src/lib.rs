//! The Platipus runtime.
//!
//! The runtime owns the pieces a compiled program needs at run time: reactive
//! state, derived values, and the event channels that custom events travel
//! through. The web target ships an equivalent implementation inside the
//! generated JavaScript; this crate is the host side of the same contract.

pub mod derived;
pub mod event;
pub mod state;

pub use derived::{Derived, DerivedSet};
pub use event::{EventBus, ListenerId};
pub use state::{Scope, StateStore, Value};

/// The state kinds a program can declare, mirroring the compiler IR.
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
            StateKind::Local => "local",
            StateKind::Shared => "shared",
            StateKind::Global => "global",
            StateKind::Persistent => "persistent",
            StateKind::Derived => "derived",
        }
    }

    pub const fn is_reactive(self) -> bool {
        matches!(self, StateKind::Local | StateKind::Shared)
    }
}

impl std::fmt::Display for StateKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
