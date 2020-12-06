//! Device path protocol

use crate::{proto::Protocol, unsafe_guid};

/// DevicePath protocol. This can be opened on a `LoadedImage.device()` handle using the `HandleProtocol` boot service. 
#[repr(C)]
#[unsafe_guid("09576e91-6d3f-11d2-8e39-00a0c969723b")]
#[derive(Protocol)]
pub struct DevicePath {
    pub device_type: DeviceType,
    pub sub_type: DeviceSubType,
    pub length: [u8; 2]
}

/// Type identifier for a DevicePath
#[repr(u8)]
#[derive(Debug)]
pub enum DeviceType {
    Hardware = 0x01,
    ACPI = 0x02,
    Messaging = 0x03,
    Media = 0x04,
    BIOSBootSpec = 0x05,
    End = 0x7F
}

/// Sub-type identifier for a DevicePath
#[repr(u8)]
#[derive(Debug)]
pub enum DeviceSubType {
    EndInstance = 0x01,
    EndEntire = 0xFF
}