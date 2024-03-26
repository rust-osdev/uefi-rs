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
use core::ffi::c_void;

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

/// Trait for creating a protocol pointer from a [`c_void`] pointer.
///
/// There is a blanket implementation for all [`Sized`] protocols that
/// simply casts the pointer to the appropriate type. Protocols that
/// are not sized must provide a custom implementation.
pub trait ProtocolPointer: Protocol {
    /// Create a const pointer to a [`Protocol`] from a [`c_void`] pointer.
    ///
    /// # Safety
    ///
    /// The input pointer must point to valid data.
    unsafe fn ptr_from_ffi(ptr: *const c_void) -> *const Self;

    /// Create a mutable pointer to a [`Protocol`] from a [`c_void`] pointer.
    ///
    /// # Safety
    ///
    /// The input pointer must point to valid data.
    unsafe fn mut_ptr_from_ffi(ptr: *mut c_void) -> *mut Self;
}

impl<P> ProtocolPointer for P
where
    P: Protocol,
{
    unsafe fn ptr_from_ffi(ptr: *const c_void) -> *const Self {
        ptr.cast::<Self>()
    }

    unsafe fn mut_ptr_from_ffi(ptr: *mut c_void) -> *mut Self {
        ptr.cast::<Self>()
    }
}

pub use uefi_macros::Protocol;

pub mod console;
pub mod debug;
pub mod device_path;
pub mod loaded_image;
pub mod media;
pub mod network;
pub mod pi;
pub mod rng;
pub mod security;
pub mod shim;
