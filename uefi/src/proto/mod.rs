// SPDX-License-Identifier: MIT OR Apache-2.0

//! High-level wrappers for [UEFI protocols].
//!
//! # TL;DR
//! Technically, a protocol is a `C` struct holding functions and/or data, with
//! an associated [`GUID`].
//!
//! # About
//! UEFI protocols are a structured collection of functions and/or data,
//! identified by a [`GUID`], which defines an interface between components in
//! the UEFI environment, such as between drivers, applications, or firmware
//! services.
//!
//! Protocols are central to UEFI’s handle-based object model, and they provide
//! a clean, extensible way for components to discover and use services from one
//! another.
//!
//! Implementation-wise, a protocol is a `C` struct holding function pointers
//! and/or data. Please note that some protocols may use [`core::ptr::null`] as
//! interface. For example, the device path protocol can be implemented but
//! return `null`.
//!
//! [`GUID`]: crate::Guid
//!
//! # More Info
//! - See the [`boot`] documentation for details of how to open a protocol.
//! - Please find additional low-level information in the
//!   [protocol section of `uefi-raw`][[UEFI protocols]].
//!
//! [`boot`]: crate::boot#accessing-protocols
//! [UEFI protocols]: uefi_raw::protocol

#[cfg(feature = "alloc")]
pub mod ata;
pub mod console;
pub mod debug;
pub mod device_path;
pub mod driver;
pub mod hii;
pub mod dma;
pub mod loaded_image;
pub mod media;
pub mod misc;
pub mod network;
#[cfg(feature = "alloc")]
pub mod nvme;
pub mod pci;
pub mod pi;
pub mod rng;
#[cfg(feature = "alloc")]
pub mod scsi;
pub mod security;
pub mod shell;
pub mod shell_params;
pub mod shim;
pub mod string;
pub mod tcg;
pub mod usb;

mod boot_policy;

pub use boot_policy::BootPolicy;
pub use uefi_macros::unsafe_protocol;

use crate::Identify;
use core::ffi::c_void;

#[cfg(doc)]
use crate::boot;

/// Marker trait for structures that represent [UEFI protocols].
///
/// Implementing this trait allows a protocol to be opened with
/// [`boot::open_protocol`] or [`boot::open_protocol_exclusive`]. Note that
/// implementing this trait does not automatically install a protocol. To
/// install a protocol, call [`boot::install_protocol_interface`].
///
/// As a convenience, you can derive the `Protocol` trait and specify the
/// protocol's GUID using the [`unsafe_protocol`] macro.
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
///
/// [UEFI protocols]: uefi_raw::protocol
pub trait Protocol: Identify {}

/// Trait for creating a [`Protocol`] pointer from a [`c_void`] pointer.
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
