use crate::{guid, Guid, Status};
use crate::protocol::device_path::DevicePathProtocol;
use core::ffi::c_void;

/// The UEFI LoadFile2 protocol.
///
/// This protocol has a single method to load a file according to some
/// device path.
///
/// This interface is implemented by many devices, e.g. network and filesystems.
#[derive(Debug)]
#[repr(C)]
pub struct LoadFile2 {
    pub load_file: unsafe extern "efiapi" fn(
        this: &mut LoadFile2,
        file_path: *const DevicePathProtocol,
        boot_policy: bool,
        buffer_size: *mut usize,
        buffer: *mut c_void,
    ) -> Status,
}

impl LoadFile2 {
    pub const GUID: Guid = guid!("4006c0c1-fcb3-403e-996d-4a6c8724e06d");
}
