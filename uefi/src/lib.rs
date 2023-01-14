//! Rusty wrapper for the Unified Extensible Firmware Interface.
//!
//! # Crate organisation
//!
//! The top-level module contains some of the most used types,
//! such as the result and error types, or other common data structures
//! such as GUIDs and handles.
//!
//! ## Tables and protocols
//!
//! The `table` module contains definitions of the UEFI tables,
//! which are structures containing some basic functions and references to other tables.
//! Most importantly, the boot services table also provides a way to obtain **protocol** handles.
//!
//! The `proto` module contains the standard UEFI protocols, which are normally provided
//! by the various UEFI drivers and firmware layers.
//!
//! ## Optional crate features
//!
//! - `alloc`: Enable functionality requiring the [`alloc`] crate from
//!   the Rust standard library. For example, methods that return a
//!   `Vec` rather than filling a statically-sized array. This requires
//!   a global allocator; you can use the `global_allocator` feature or
//!   provide your own.
//! - `global_allocator`: Implement a [global allocator] using UEFI
//!   functions. This is a simple allocator that relies on the UEFI pool
//!   allocator. You can choose to provide your own allocator instead of
//!   using this feature, or no allocator at all if you don't need to
//!   dynamically allocate any memory.
//! - `logger`: Logging implementation for the standard [`log`] crate
//!   that prints output to the UEFI console. No buffering is done; this
//!   is not a high-performance logger.
//! - `panic-on-logger-errors` (enabled by default): Panic if a text
//!   output error occurs in the logger.
//! - `unstable`: Enable functionality that depends on [unstable
//!   features] in the nightly compiler. Note that currently the `uefi`
//!   crate _always_ requires unstable features even if the `unstable`
//!   feature is not enabled, but once a couple more required features
//!   are stabilized we intend to make the `uefi` crate work on the
//!   stable channel by default.
//!   As example, in conjunction with the `alloc`-feature, this gate allows
//!   the `allocator_api` on certain functions.
//!
//! The `global_allocator` and `logger` features require special
//! handling to perform initialization and tear-down. The
//! [`uefi-services`] crate provides an `init` method that takes care of
//! this.
//!
//! ## Adapting to local conditions
//!
//! Unlike system tables, which are present on *all* UEFI implementations,
//! protocols *may* or *may not* be present on a certain system.
//!
//! For example, a PC with no network card might not contain a network driver,
//! therefore all the network protocols will be unavailable.
//!
//! [`GlobalAlloc`]: alloc::alloc::GlobalAlloc
//! [`uefi-services`]: https://crates.io/crates/uefi-services
//! [unstable features]: https://doc.rust-lang.org/unstable-book/

#![cfg_attr(feature = "unstable", feature(error_in_core))]
#![cfg_attr(all(feature = "unstable", feature = "alloc"), feature(allocator_api))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![no_std]
// Enable some additional warnings and lints.
#![warn(clippy::ptr_as_ptr, missing_docs, unused)]
#![deny(clippy::all)]
#![deny(clippy::must_use_candidate)]

#[cfg(feature = "alloc")]
extern crate alloc;

// allow referring to self as ::uefi for macros to work universally (from this crate and from others)
// see https://github.com/rust-lang/rust/issues/54647
extern crate self as uefi;

#[macro_use]
pub mod data_types;
#[cfg(feature = "alloc")]
pub use self::data_types::CString16;
pub use self::data_types::Identify;
pub use self::data_types::{CStr16, CStr8, Char16, Char8, Event, Guid, Handle};
pub use uefi_macros::guid;

mod result;
pub use self::result::{Error, Result, ResultExt, Status};

pub mod table;

pub mod proto;

pub mod prelude;

#[cfg(feature = "global_allocator")]
pub mod global_allocator;

#[cfg(feature = "logger")]
pub mod logger;

// As long as this is behind "alloc", we can simplify cfg-feature attributes in this module.
#[cfg(feature = "alloc")]
pub(crate) mod mem;

pub(crate) mod polyfill;
