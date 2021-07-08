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
use core::slice;

/// Header that appears at the start of every [`DevicePath`] node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C, packed)]
pub struct DevicePathHeader {
    /// Type of device
    pub device_type: DeviceType,
    /// Sub type of device
    pub sub_type: DeviceSubType,
    /// Size (in bytes) of the full [`DevicePath`] instance, including this header.
    pub length: u16,
}

/// Device path protocol.
///
/// This can be opened on a `LoadedImage.device()` handle using the `HandleProtocol` boot service.
#[repr(C, packed)]
#[unsafe_guid("09576e91-6d3f-11d2-8e39-00a0c969723b")]
#[derive(Eq, Protocol)]
pub struct DevicePath {
    header: DevicePathHeader,
}

impl DevicePath {
    /// Type of device
    pub fn device_type(&self) -> DeviceType {
        self.header.device_type
    }

    /// Sub type of device
    pub fn sub_type(&self) -> DeviceSubType {
        self.header.sub_type
    }

    /// Size (in bytes) of the full [`DevicePath`] instance, including the header.
    pub fn length(&self) -> u16 {
        self.header.length
    }

    /// True if this node ends the entire path.
    pub fn is_end_entire(&self) -> bool {
        self.device_type() == DeviceType::END && self.sub_type() == DeviceSubType::END_ENTIRE
    }

    /// Get an iterator over the [`DevicePath`] nodes starting at
    /// `self`. Iteration ends when a path is reached where
    /// [`is_end_entire`][DevicePath::is_end_entire] is true. That ending path
    /// is not returned by the iterator.
    pub fn iter(&self) -> DevicePathIterator {
        DevicePathIterator { path: self }
    }
}

impl PartialEq for DevicePath {
    fn eq(&self, other: &DevicePath) -> bool {
        // Check for equality with a byte-by-byte comparison of the device
        // paths. Note that this covers the entire payload of the device path
        // using the `length` field in the header, so it's not the same as just
        // comparing the fields of the `DevicePath` struct.
        unsafe {
            let self_bytes = slice::from_raw_parts(
                self as *const DevicePath as *const u8,
                self.length() as usize,
            );
            let other_bytes = slice::from_raw_parts(
                other as *const DevicePath as *const u8,
                other.length() as usize,
            );

            self_bytes == other_bytes
        }
    }
}

/// Iterator over [`DevicePath`] nodes.
///
/// Iteration ends when a path is reached where [`DevicePath::is_end_entire`]
/// is true. That ending path is not returned by the iterator.
///
/// This struct is returned by [`DevicePath::iter`].
pub struct DevicePathIterator<'a> {
    path: &'a DevicePath,
}

impl<'a> Iterator for DevicePathIterator<'a> {
    type Item = &'a DevicePath;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.path;

        if cur.is_end_entire() {
            return None;
        }

        // Advance self.path to the next entry.
        let len = cur.header.length;
        let byte_ptr = cur as *const DevicePath as *const u8;
        unsafe {
            let next_path_ptr = byte_ptr.add(len as usize) as *const DevicePath;
            self.path = &*next_path_ptr;
        }

        Some(cur)
    }
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

/// Sub-type identifier for a DevicePath
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeviceSubType(pub u8);

impl DeviceSubType {
    /// PCI Device Path.
    pub const HARDWARE_PCI: DeviceSubType = DeviceSubType(0x01);
    /// PCCARD Device Path.
    pub const HARDWARE_PCCARD: DeviceSubType = DeviceSubType(0x02);
    /// Memory-mapped Device Path.
    pub const HARDWARE_MEMORY_MAPPED: DeviceSubType = DeviceSubType(0x03);
    /// Vendor-Defined Device Path.
    pub const HARDWARE_VENDOR: DeviceSubType = DeviceSubType(0x04);
    /// Controller Device Path.
    pub const HARDWARE_CONTROLLER: DeviceSubType = DeviceSubType(0x05);
    /// BMC Device Path.
    pub const HARDWARE_BMC: DeviceSubType = DeviceSubType(0x06);

    /// ACPI Device Path.
    pub const ACPI: DeviceSubType = DeviceSubType(0x01);
    /// Expanded ACPI Device Path.
    pub const ACPI_EXPANDED: DeviceSubType = DeviceSubType(0x02);
    /// ACPI _ADR Device Path.
    pub const ACPI_ADR: DeviceSubType = DeviceSubType(0x03);
    /// NVDIMM Device Path.
    pub const ACPI_NVDIMM: DeviceSubType = DeviceSubType(0x04);

    /// ATAPI Device Path.
    pub const MESSAGING_ATAPI: DeviceSubType = DeviceSubType(0x01);
    /// SCSI Device Path.
    pub const MESSAGING_SCSI: DeviceSubType = DeviceSubType(0x02);
    /// Fibre Channel Device Path.
    pub const MESSAGING_FIBRE_CHANNEL: DeviceSubType = DeviceSubType(0x03);
    /// 1394 Device Path.
    pub const MESSAGING_1394: DeviceSubType = DeviceSubType(0x04);
    /// USB Device Path.
    pub const MESSAGING_USB: DeviceSubType = DeviceSubType(0x05);
    /// I2O Device Path.
    pub const MESSAGING_I2O: DeviceSubType = DeviceSubType(0x06);
    /// Infiniband Device Path.
    pub const MESSAGING_INFINIBAND: DeviceSubType = DeviceSubType(0x09);
    /// Vendor-Defined Device Path.
    pub const MESSAGING_VENDOR: DeviceSubType = DeviceSubType(0x0a);
    /// MAC Address Device Path.
    pub const MESSAGING_MAC_ADDRESS: DeviceSubType = DeviceSubType(0x0b);
    /// IPV4 Device Path.
    pub const MESSAGING_IPV4: DeviceSubType = DeviceSubType(0x0c);
    /// IPV6 Device Path.
    pub const MESSAGING_IPV6: DeviceSubType = DeviceSubType(0x0d);
    /// UART Device Path.
    pub const MESSAGING_UART: DeviceSubType = DeviceSubType(0x0e);
    /// USB Class Device Path.
    pub const MESSAGING_USB_CLASS: DeviceSubType = DeviceSubType(0x0f);
    /// USB WWID Device Path.
    pub const MESSAGING_USB_WWID: DeviceSubType = DeviceSubType(0x10);
    /// Device Logical Unit.
    pub const MESSAGING_DEVICE_LOGICAL_UNIT: DeviceSubType = DeviceSubType(0x11);
    /// SATA Device Path.
    pub const MESSAGING_SATA: DeviceSubType = DeviceSubType(0x12);
    /// iSCSI Device Path node (base information).
    pub const MESSAGING_ISCSI: DeviceSubType = DeviceSubType(0x13);
    /// VLAN Device Path node.
    pub const MESSAGING_VLAN: DeviceSubType = DeviceSubType(0x14);
    /// Fibre Channel Ex Device Path.
    pub const MESSAGING_FIBRE_CHANNEL_EX: DeviceSubType = DeviceSubType(0x15);
    /// Serial Attached SCSI (SAS) Ex Device Path.
    pub const MESSAGING_SCSI_SAS_EX: DeviceSubType = DeviceSubType(0x16);
    /// NVM Express Namespace Device Path.
    pub const MESSAGING_NVME_NAMESPACE: DeviceSubType = DeviceSubType(0x17);
    /// Uniform Resource Identifiers (URI) Device Path.
    pub const MESSAGING_URI: DeviceSubType = DeviceSubType(0x18);
    /// UFS Device Path.
    pub const MESSAGING_UFS: DeviceSubType = DeviceSubType(0x19);
    /// SD (Secure Digital) Device Path.
    pub const MESSAGING_SD: DeviceSubType = DeviceSubType(0x1a);
    /// Bluetooth Device Path.
    pub const MESSAGING_BLUETOOTH: DeviceSubType = DeviceSubType(0x1b);
    /// Wi-Fi Device Path.
    pub const MESSAGING_WIFI: DeviceSubType = DeviceSubType(0x1c);
    /// eMMC (Embedded Multi-Media Card) Device Path.
    pub const MESSAGING_EMMC: DeviceSubType = DeviceSubType(0x1d);
    /// BluetoothLE Device Path.
    pub const MESSAGING_BLUETOOTH_LE: DeviceSubType = DeviceSubType(0x1e);
    /// DNS Device Path.
    pub const MESSAGING_DNS: DeviceSubType = DeviceSubType(0x1f);
    /// NVDIMM Namespace Device Path.
    pub const MESSAGING_NVDIMM_NAMESPACE: DeviceSubType = DeviceSubType(0x20);

    /// Hard Drive Media Device Path.
    pub const MEDIA_HARD_DRIVE: DeviceSubType = DeviceSubType(0x01);
    /// CD-ROM Media Device Path.
    pub const MEDIA_CD_ROM: DeviceSubType = DeviceSubType(0x02);
    /// Vendor-Defined Media Device Path.
    pub const MEDIA_VENDOR: DeviceSubType = DeviceSubType(0x03);
    /// File Path Media Device Path.
    pub const MEDIA_FILE_PATH: DeviceSubType = DeviceSubType(0x04);
    /// Media Protocol Device Path.
    pub const MEDIA_PROTOCOL: DeviceSubType = DeviceSubType(0x05);
    /// PIWG Firmware File.
    pub const MEDIA_PIWG_FIRMWARE_FILE: DeviceSubType = DeviceSubType(0x06);
    /// PIWG Firmware Volume.
    pub const MEDIA_PIWG_FIRMWARE_VOLUME: DeviceSubType = DeviceSubType(0x07);
    /// Relative Offset Range.
    pub const MEDIA_RELATIVE_OFFSET_RANGE: DeviceSubType = DeviceSubType(0x08);
    /// RAM Disk Device Path.
    pub const MEDIA_RAM_DISK: DeviceSubType = DeviceSubType(0x09);

    /// BIOS Boot Specification Device Path.
    pub const BIOS_BOOT_SPECIFICATION: DeviceSubType = DeviceSubType(0x01);

    /// End this instance of a Device Path and start a new one.
    pub const END_INSTANCE: DeviceSubType = DeviceSubType(0x01);
    /// End entire Device Path.
    pub const END_ENTIRE: DeviceSubType = DeviceSubType(0xff);
}

/// ACPI Device Path
#[repr(C, packed)]
pub struct AcpiDevicePath {
    header: DevicePathHeader,

    /// Device's PnP hardware ID stored in a numeric 32-bit compressed EISA-type ID. This value must match the
    /// corresponding _HID in the ACPI name space.
    pub hid: u32,
    /// Unique ID that is required by ACPI if two devices have the same _HID. This value must also match the
    /// corresponding _UID/_HID pair in the ACPI name space. Only the 32-bit numeric value type of _UID is supported;
    /// thus strings must not be used for the _UID in the ACPI name space.
    pub uid: u32,
}
