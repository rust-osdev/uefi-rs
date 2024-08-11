use crate::{guid, Boolean, Char16, Guid};

/// Device path protocol.
///
/// A device path contains one or more device path instances made of up
/// variable-length nodes.
///
/// Note that the fields in this struct define the header at the start of each
/// node; a device path is typically larger than these four bytes.
#[derive(Debug)]
#[repr(C)]
pub struct DevicePathProtocol {
    pub major_type: u8,
    pub sub_type: u8,
    pub length: [u8; 2],
    // followed by payload (dynamically sized)
}

impl DevicePathProtocol {
    pub const GUID: Guid = guid!("09576e91-6d3f-11d2-8e39-00a0c969723b");
}

#[derive(Debug)]
#[repr(C)]
pub struct DevicePathToTextProtocol {
    pub convert_device_node_to_text: unsafe extern "efiapi" fn(
        device_node: *const DevicePathProtocol,
        display_only: Boolean,
        allow_shortcuts: Boolean,
    ) -> *const Char16,
    pub convert_device_path_to_text: unsafe extern "efiapi" fn(
        device_path: *const DevicePathProtocol,
        display_only: Boolean,
        allow_shortcuts: Boolean,
    ) -> *const Char16,
}

impl DevicePathToTextProtocol {
    pub const GUID: Guid = guid!("8b843e20-8132-4852-90cc-551a4e4a7f1c");
}

#[derive(Debug)]
#[repr(C)]
pub struct DevicePathFromTextProtocol {
    pub convert_text_to_device_node:
        unsafe extern "efiapi" fn(text_device_node: *const Char16) -> *const DevicePathProtocol,
    pub convert_text_to_device_path:
        unsafe extern "efiapi" fn(text_device_path: *const Char16) -> *const DevicePathProtocol,
}

impl DevicePathFromTextProtocol {
    pub const GUID: Guid = guid!("05c99a21-c70f-4ad2-8a5f-35df3343f51e");
}
