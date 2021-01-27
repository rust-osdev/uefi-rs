//! Device path protocol

use crate::{proto::Protocol, unsafe_guid};

use uefi_sys::EFI_DEVICE_PATH_PROTOCOL;

/// DevicePath protocol. This can be opened on a `LoadedImage.device()` handle
/// using the `HandleProtocol` boot service.
#[repr(C)]
#[unsafe_guid("09576e91-6d3f-11d2-8e39-00a0c969723b")]
#[derive(Protocol)]
pub struct DevicePath {
    /// Unsafe raw type extracted from EDK2
    pub raw: EFI_DEVICE_PATH_PROTOCOL,
}

impl DevicePath {
    /// Type of device
    pub fn device_type(&self) -> DeviceType {
        unsafe { core::mem::transmute(self.raw.Type) }
    }

    /// Sub type of device
    pub fn sub_type(&self) -> DeviceSubType {
        unsafe { core::mem::transmute(self.raw.SubType) }
    }

    /// Data related to device path
    ///
    /// The device_type and sub_type determine the
    /// kind of data, and it size.
    pub fn length(&self) -> [u8; 2] {
        self.raw.Length
    }
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
    End = 0x7F,
}

/// Sub-type identifier for a DevicePath
#[repr(u8)]
#[derive(Debug)]
pub enum DeviceSubType {
    EndInstance = 0x01,
    EndEntire = 0xFF,
}
