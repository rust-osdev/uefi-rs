use core::ffi::c_void;

/// Opaque handle to an UEFI entity (protocol, image...)
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Handle(*mut c_void);

/// Handle to an event structure
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Event(*mut c_void);

mod guid;
pub use self::guid::Guid;

#[macro_use]
mod enums;
