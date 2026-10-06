//! The Platipus runtime facade.
//!
//! The reactive contract lives in `platipus-reactive`; this crate re-exports
//! it so existing consumers of `platipus-runtime` keep working.

pub use platipus_reactive::derived;
pub use platipus_reactive::event;
pub use platipus_reactive::state;
pub use platipus_reactive::*;
