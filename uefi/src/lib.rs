//! Rusty wrapper for the [Unified Extensible Firmware Interface][UEFI].
//!
//! See the [Rust UEFI Book] for a tutorial, how-tos, and overviews of some
//! important UEFI concepts. For more details of UEFI, see the latest [UEFI
//! Specification][spec].
//!
//! Feel free to file bug reports and questions in our [issue tracker], and [PR
//! contributions][contributing] are also welcome!
//!
//! # Crate organisation
//!
//! The top-level module contains some of the most used types and macros,
//! including the [`Handle`] and [`Result`] types, the [`CStr16`] and
//! [`CString16`] types for working with UCS-2 strings, and the [`entry`] and
//! [`guid`] macros.
//!
//! ## Tables
//!
//! The [`SystemTable`] provides access to almost everything in UEFI. It comes
//! in two flavors:
//! - `SystemTable<Boot>`: for boot-time applications such as bootloaders,
//!   provides access to both boot and runtime services.
//! - `SystemTable<Runtime>`: for operating systems after boot services have
//!   been exited.
//!
//! ## Protocols
//!
//! When boot services are active, most functionality is provided via UEFI
//! protocols. Protocols provide operations such as reading and writing files,
//! drawing to the screen, sending and receiving network requests, and much
//! more. The list of protocols that are actually available when running an
//! application depends on the device. For example, a PC with no network card
//! may not provide network protocols.
//!
//! See the [`BootServices`] documentation for details of how to open a
//! protocol, and see the [`proto`] module for protocol implementations. New
//! protocols can be defined with the [`unsafe_protocol`] macro.
//!
//! ## Optional crate features
//!
//! - `alloc`: Enable functionality requiring the [`alloc`] crate from
//!   the Rust standard library. For example, methods that return a
//!   `Vec` rather than filling a statically-sized array. This requires
//!   a global allocator; you can use the `global_allocator` feature or
//!   provide your own.
//! - `global_allocator`: Set [`allocator::Allocator`] as the global Rust
//!   allocator. This is a simple allocator that relies on the UEFI pool
//!   allocator. You can choose to provide your own allocator instead of
//!   using this feature, or no allocator at all if you don't need to
//!   dynamically allocate any memory.
//! - `logger`: Logging implementation for the standard [`log`] crate
//!   that prints output to the UEFI console. No buffering is done; this
//!   is not a high-performance logger.
//! - `panic-on-logger-errors` (enabled by default): Panic if a text
//!   output error occurs in the logger.
//! - `unstable`: Enable functionality that depends on [unstable
//!   features] in the nightly compiler.
//!   As example, in conjunction with the `alloc`-feature, this gate allows
//!   the `allocator_api` on certain functions.
//!
//! The `global_allocator` and `logger` features require special
//! handling to perform initialization and tear-down. The
//! [`uefi-services`] crate provides an `init` method that takes care of
//! this.
//!
//! [Rust UEFI Book]: https://rust-osdev.github.io/uefi-rs/HEAD/
//! [UEFI]: https://uefi.org/
//! [`BootServices`]: table::boot::BootServices
//! [`GlobalAlloc`]: alloc::alloc::GlobalAlloc
//! [`SystemTable`]: table::SystemTable
//! [`uefi-services`]: https://crates.io/crates/uefi-services
//! [`unsafe_protocol`]: proto::unsafe_protocol
//! [contributing]: https://github.com/rust-osdev/uefi-rs/blob/main/CONTRIBUTING.md
//! [issue tracker]: https://github.com/rust-osdev/uefi-rs/issues
//! [spec]: https://uefi.org/specifications
//! [unstable features]: https://doc.rust-lang.org/unstable-book/

#![cfg_attr(feature = "unstable", feature(error_in_core))]
#![cfg_attr(all(feature = "unstable", feature = "alloc"), feature(allocator_api))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![no_std]
// Enable some additional warnings and lints.
#![warn(clippy::ptr_as_ptr, missing_docs, unused)]
#![deny(clippy::all)]
#![deny(clippy::must_use_candidate)]
#![deny(missing_debug_implementations)]

#[cfg(feature = "alloc")]
extern crate alloc;

// allow referring to self as ::uefi for macros to work universally (from this crate and from others)
// see https://github.com/rust-lang/rust/issues/54647
extern crate self as uefi;

#[macro_use]
pub mod data_types;
#[cfg(feature = "alloc")]
pub use self::data_types::CString16;
pub use self::data_types::{CStr16, CStr8, Char16, Char8, Event, Guid, Handle, Identify};
pub use uefi_macros::{cstr16, cstr8, entry, guid};

mod result;
pub use self::result::{Error, Result, ResultExt, Status};

pub mod table;

pub mod proto;

pub mod prelude;

pub mod allocator;

#[cfg(feature = "logger")]
pub mod logger;

#[cfg(feature = "alloc")]
pub mod fs;

// As long as this is behind "alloc", we can simplify cfg-feature attributes in this module.
#[cfg(feature = "alloc")]
pub(crate) mod mem;

pub(crate) mod polyfill;

mod util;
