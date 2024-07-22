//! Types, functions, traits, and other helpers to work with memory in UEFI
//! libraries and applications.

pub mod memory_map;
#[cfg(feature = "alloc")]
pub(crate) mod util;

#[cfg(feature = "alloc")]
pub(crate) use util::*;
