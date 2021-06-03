//! `DevicePath` protocol
//!
//! Device Paths are a packed array of Device Path Nodes. Each Node
//! immediately follows the previous one, and each node may appear on
//! any byte boundary. The array must be terminated with an End of
//! Hardware Device Path Node.
//!
//! Device Path Nodes are variable-length structures that can represent
//! different types of paths. For example, a File Path Media Device
//! Path contains a typical Windows-style path such as
//! "\efi\boot\bootx64.efi", whereas an ACPI Device Path contains
//! numeric ACPI IDs.
//!
//! A Device Path Node always starts with the `DevicePath` header. The
//! `device_type` and `sub_type` fields determine the type of data in
//! the rest of the structure, and the `length` field indicates the
//! total size of the Node including the header.

use crate::{proto::Protocol, unsafe_guid};

/// Device path protocol.
///
/// This can be opened on a `LoadedImage.device()` handle using the `HandleProtocol` boot service.
#[repr(C, packed)]
#[unsafe_guid("09576e91-6d3f-11d2-8e39-00a0c969723b")]
#[derive(Protocol)]
pub struct DevicePath {
    /// Type of device
    pub device_type: DeviceType,
    /// Sub type of device
    pub sub_type: DeviceSubType,
    /// Data related to device path
    ///
    /// The `device_type` and `sub_type` determine the kind of data, and its size.
    pub length: [u8; 2],
}

newtype_enum! {
/// Type identifier for a DevicePath
pub enum DeviceType: u8 => {
    /// Hardware Device Path.
    ///
    /// This Device Path defines how a device is attached to the resource domain of a system, where resource domain is
    /// simply the shared memory, memory mapped I/ O, and I/O space of the system.
    HARDWARE = 0x01,
    /// ACPI Device Path.
    ///
    /// This Device Path is used to describe devices whose enumeration is not described in an industry-standard fashion.
    /// These devices must be described using ACPI AML in the ACPI namespace; this Device Path is a linkage to the ACPI
    /// namespace.
    ACPI = 0x02,
    /// Messaging Device Path.
    ///
    /// This Device Path is used to describe the connection of devices outside the resource domain of the system. This
    /// Device Path can describe physical messaging information such as a SCSI ID, or abstract information such as
    /// networking protocol IP addresses.
    MESSAGING = 0x03,
    /// Media Device Path.
    ///
    /// This Device Path is used to describe the portion of a medium that is being abstracted by a boot service.
    /// For example, a Media Device Path could define which partition on a hard drive was being used.
    MEDIA = 0x04,
    /// BIOS Boot Specification Device Path.
    ///
    /// This Device Path is used to point to boot legacy operating systems; it is based on the BIOS Boot Specification
    /// Version 1.01.
    BIOS_BOOT_SPEC = 0x05,
    /// End of Hardware Device Path.
    ///
    /// Depending on the Sub-Type, this Device Path node is used to indicate the end of the Device Path instance or
    /// Device Path structure.
    END = 0x7F,
}}

newtype_enum! {
/// Sub-type identifier for a DevicePath
pub enum DeviceSubType: u8 => {
    /// End This Instance of a Device Path and start a new Device Path
    END_INSTANCE = 0x01,
    /// End Entire Device Path
    END_ENTIRE = 0xFF,
}}

/// ACPI Device Path
#[repr(C, packed)]
pub struct AcpiDevicePath {
    /// Type of device, which is ACPI Device Path
    pub device_type: DeviceType,
    /// Sub type of the device, which is ACPI Device Path
    pub sub_type: DeviceSubType,
    /// Device's PnP hardware ID stored in a numeric 32-bit compressed EISA-type ID. This value must match the
    /// corresponding _HID in the ACPI name space.
    pub hid: u32,
    /// Unique ID that is required by ACPI if two devices have the same _HID. This value must also match the
    /// corresponding _UID/_HID pair in the ACPI name space. Only the 32-bit numeric value type of _UID is supported;
    /// thus strings must not be used for the _UID in the ACPI name space.
    pub uid: u32,
}
