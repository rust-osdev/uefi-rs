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
//! ## Adapting to local conditions
//!
//! Unlike system tables, which are present on *all* UEFI implementations,
//! protocols *may* or *may not* be present on a certain system.
//!
//! For example, a PC with no network card might not contain a network driver,
//! therefore all the network protocols will be unavailable.

#![feature(optin_builtin_traits)]
#![feature(min_const_fn)]
#![feature(tool_lints)]
#![feature(try_trait)]
#![no_std]
// Enable some additional warnings and lints.
#![warn(missing_docs, unused)]
#![deny(clippy::all)]

mod error;
pub use self::error::{Result, status, Status};

mod data_types;
pub use self::data_types::{Event, Guid, Handle};

pub mod table;

pub mod proto;

pub mod prelude;
