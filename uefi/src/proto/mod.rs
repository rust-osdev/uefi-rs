//! Protocol definitions.
//!
//! Protocols are sets of related functionality identified by a unique
//! ID. They can be implemented by a UEFI driver or occasionally by a
//! UEFI application.
//!
//! See the [`boot`] documentation for details of how to open a protocol.
//!
//! [`boot`]: crate::boot#accessing-protocols

pub mod console;
pub mod debug;
pub mod device_path;
pub mod driver;
pub mod loaded_image;
pub mod media;
pub mod misc;
pub mod network;
pub mod pi;
pub mod rng;
pub mod security;
pub mod shell_params;
pub mod shim;
pub mod string;
pub mod tcg;

mod boot_policy;

pub use boot_policy::BootPolicy;
pub use uefi_macros::unsafe_protocol;

use crate::Identify;
use core::ffi::c_void;

/// Common trait implemented by all standard UEFI protocols.
///
/// You can derive the `Protocol` trait and specify the protocol's GUID using
/// the [`unsafe_protocol`] macro.
///
/// # Example
///
/// ```
/// use uefi::{Identify, guid};
/// use uefi::proto::unsafe_protocol;
///
/// #[unsafe_protocol("12345678-9abc-def0-1234-56789abcdef0")]
/// struct ExampleProtocol {}
///
/// assert_eq!(ExampleProtocol::GUID, guid!("12345678-9abc-def0-1234-56789abcdef0"));
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
