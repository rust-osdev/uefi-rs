//! Console support protocols.
//!
//! The console represents the various input and output methods
//! used by the user to interact with the early boot platform.

pub mod gop;
pub mod pointer;
pub mod serial;
pub mod text;
#[cfg(feature = "draw_target")]
pub mod draw_target;
