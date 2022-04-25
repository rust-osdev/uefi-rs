//! Protocol definitions.
//!
//! Protocols are sets of related functionality identified by a unique
//! ID. They can be implemented by a UEFI driver or occasionally by a
//! UEFI application.
//!
//! See the [`BootServices`] documentation for details of how to open a
//! protocol.
//!
//! [`BootServices`]: crate::table::boot::BootServices#accessing-protocols

use crate::Identify;

/// Common trait implemented by all standard UEFI protocols
///
/// According to the UEFI's specification, protocols are `!Send` (they expect to
/// be run on the bootstrap processor) and `!Sync` (they are not thread-safe).
/// You can derive the `Protocol` trait, add these bounds and specify the
/// protocol's GUID using the following syntax:
///
/// ```
/// #![feature(negative_impls)]
/// use uefi::{proto::Protocol, unsafe_guid};
/// #[unsafe_guid("12345678-9abc-def0-1234-56789abcdef0")]
/// #[derive(Protocol)]
/// struct DummyProtocol {}
/// ```
pub trait Protocol: Identify {}

pub use uefi_macros::Protocol;

pub mod console;
pub mod debug;
pub mod device_path;
pub mod loaded_image;
pub mod media;
pub mod pi;
pub mod rng;
pub mod shim;
