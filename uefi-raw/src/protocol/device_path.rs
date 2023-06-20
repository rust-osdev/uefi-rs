use crate::{guid, Guid};

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
