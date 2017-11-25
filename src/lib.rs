//! Rusty wrapper for the Unified Extensible Firmware Interface.

#![feature(try_trait)]
#![feature(optin_builtin_traits)]
#![feature(const_fn)]
#![feature(conservative_impl_trait)]

#![no_std]

#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", warn(clippy))]

#[macro_use]
extern crate bitflags;

mod guid;
pub use self::guid::Guid;

mod status;
pub use self::status::Status;

use core::result;
/// Return type of many UEFI functions.
pub type Result<T> = result::Result<T, Status>;

/// A pointer to an opaque data structure.
pub type Handle = *mut ();

pub mod table;

pub mod proto;
