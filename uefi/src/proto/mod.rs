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

use crate::{Guid, Identify};
use alloc::boxed::Box;
use core::ffi::c_void;
use core::marker::PhantomData;

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
pub trait Protocol: Identify {
    /// Optional GUID for Service Binding Protocol, when applicable.
    const SERVICE_BINDING: Option<Guid> = None;

    /// Raw FFI protocol interface.
    type Raw: RawProtocol;
}

/// Trait for raw protocol interfaces to wrap themselves in a struct that can
/// have its own state.
pub trait RawProtocol {
    /// When wrapping the protocol with another (non-transparent repr) struct,
    /// this function can be implemented to initialize the struct and turn it
    /// back into a raw pointer. This is usually implemented with a call to
    /// [`Box::into_raw`].
    ///
    /// Note that the null pointer case must be handled.
    fn wrap(ptr: *mut c_void) -> *mut c_void;

    /// Callback used in the drop function for [`ScopedProtocol`] to cleanup any
    /// memory other than the wrapped protocol (from FFI). This is usually implemented
    /// with a call to [`Box::from_raw`].
    fn drop_wrapper(ptr: *mut c_void);
}

/// NoWrapper does not mutate or track the protocol pointer from FFI in any way.
#[derive(Debug)]
pub struct NoWrapper {}

impl RawProtocol for NoWrapper {
    fn wrap(ptr: *mut c_void) -> *mut c_void {
        ptr
    }

    fn drop_wrapper(_ptr: *mut c_void) {}
}

/// Helper struct to implement [`RawProtocol`].
#[derive(Debug)]
pub struct StructWrapper<Inner, Wrapper> {
    inner_type: PhantomData<Inner>,
    wrapper_type: PhantomData<Wrapper>,
}

impl<Inner, Wrapper> RawProtocol for StructWrapper<Inner, Wrapper>
where
    Wrapper: From<*mut Inner>,
{
    fn wrap(ptr: *mut c_void) -> *mut c_void {
        if ptr.is_null() {
            return ptr;
        }

        let raw: *mut Inner = ptr.cast();
        let wrapped: Wrapper = raw.into();
        Box::into_raw(Box::new(wrapped)) as *mut c_void
    }

    fn drop_wrapper(ptr: *mut c_void) {
        if ptr.is_null() {
            return;
        }

        unsafe { drop(Box::from_raw(ptr as *mut Wrapper)) }
    }
}

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

pub use uefi_macros::unsafe_protocol;

pub mod console;
pub mod debug;
pub mod device_path;
pub mod driver;
pub mod loaded_image;
pub mod media;
pub mod network;
pub mod pi;
pub mod rng;
pub mod security;
pub mod shim;
pub mod string;
pub mod tcg;
