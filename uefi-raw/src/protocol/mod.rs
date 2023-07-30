//! Protocol definitions.
//!
//! Protocols are sets of related functionality identified by a unique
//! ID. They can be implemented by a UEFI driver or occasionally by a
//! UEFI application.

pub mod console;
pub mod device_path;
pub mod disk;
pub mod loaded_image;
pub mod rng;

use crate::{Handle, Status};

#[repr(C)]
pub struct ServiceBinding {
    pub create_child:
        unsafe extern "efiapi" fn(this: *mut Self, child_handle: *mut Handle) -> Status,
    pub destroy_child: unsafe extern "efiapi" fn(this: *mut Self, child_handle: Handle) -> Status,
}
