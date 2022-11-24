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
//! ## Optional crate features:
//!
//! - `alloc`: Enables functionality requiring the `alloc` crate from the Rust standard library.
//!   - For example, this allows many convenient `uefi-rs` functions to operate on heap data (`Box`).
//!   - It is up to the user to provide a `#[global_allocator]`.
//! - `global_allocator`: implements a `#[global_allocator]` using UEFI functions.
//!   - This allows you to use all abstractions from the `alloc` crate from the Rust standard library
//!     during runtime. Hence, `Vec`, `Box`, etc. will be able to allocate memory.
//!     **This is optional**, so you can provide a custom `#[global_allocator]` as well.
//!   - There's no guarantee of the efficiency of UEFI's allocator.
//! - `logger`: logging implementation for the standard [`log`] crate.
//!   - Prints output to UEFI console.
//!   - No buffering is done: this is not a high-performance logger.
//!
//! ## Adapting to local conditions
//!
//! Unlike system tables, which are present on *all* UEFI implementations,
//! protocols *may* or *may not* be present on a certain system.
//!
//! For example, a PC with no network card might not contain a network driver,
//! therefore all the network protocols will be unavailable.

#![feature(abi_efiapi)]
#![feature(maybe_uninit_slice)]
#![feature(negative_impls)]
#![feature(ptr_metadata)]
#![cfg_attr(feature = "alloc", feature(vec_into_raw_parts))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![no_std]
// Enable some additional warnings and lints.
#![warn(clippy::ptr_as_ptr, missing_docs, unused)]
#![deny(clippy::all)]

// Enable once we use vec![] or similar
// #[cfg_attr(feature = "alloc", macro_use)]
#[cfg(feature = "alloc")]
extern crate alloc;

// allow referring to self as ::uefi for macros to work universally (from this crate and from others)
// see https://github.com/rust-lang/rust/issues/54647
extern crate self as uefi;

#[macro_use]
pub mod data_types;
#[cfg(feature = "alloc")]
pub use self::data_types::CString16;
pub use self::data_types::{unsafe_guid, Identify};
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

#[cfg(feature = "alloc")]
pub(crate) mod mem;
