use crate::protocol::device_path::DevicePathProtocol;
use crate::table::boot::MemoryType;
use crate::table::system::SystemTable;
use crate::{guid, Guid, Handle, Status};
use core::ffi::c_void;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct LoadedImageProtocol {
    pub revision: u32,
    pub parent_handle: Handle,
    pub system_table: *const SystemTable,

    // Source location of the image.
    pub device_handle: Handle,
    pub file_path: *const DevicePathProtocol,

    pub reserved: *const c_void,

    // Image load options.
    pub load_options_size: u32,
    pub load_options: *const c_void,

    // Location where image was loaded.
    pub image_base: *const c_void,
    pub image_size: u64,
    pub image_code_type: MemoryType,
    pub image_data_type: MemoryType,
    pub unload: Option<unsafe extern "efiapi" fn(image_handle: Handle) -> Status>,
}

impl LoadedImageProtocol {
    pub const GUID: Guid = guid!("5b1b31a1-9562-11d2-8e3f-00a0c969723b");
}
