//! Raw interface for working with UEFI.
//!
//! This crate is intended for implementing UEFI services. It is also used for
//! implementing the [`uefi`] crate, which provides a safe wrapper around UEFI.
//!
//! For creating UEFI applications and drivers, consider using the [`uefi`]
//! crate instead of `uefi-raw`.
//!
//! [`uefi`]: https://crates.io/crates/uefi

#![no_std]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(clippy::all)]
#![deny(clippy::ptr_as_ptr, unused)]
#![deny(clippy::must_use_candidate)]

#[macro_use]
mod enums;
