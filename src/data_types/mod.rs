use core::ffi::c_void;

/// A collection of related interfaces
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Handle(*mut c_void);

/// Handle to an event structure
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Event(*mut c_void);

mod guid;
pub use self::guid::Guid;
