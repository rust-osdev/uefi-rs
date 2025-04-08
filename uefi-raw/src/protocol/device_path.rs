// SPDX-License-Identifier: MIT OR Apache-2.0

mod device_path_gen;

use crate::{guid, Boolean, Char16, Guid};

pub use device_path_gen::{acpi, bios_boot_spec, end, hardware, media, messaging};

/// Device path protocol.
///
/// A device path contains one or more device path instances made up of
/// variable-length nodes.
///
/// Note that the fields in this struct define the header at the start of each
/// node; a device path is typically larger than these four bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(C)]
pub struct DevicePathProtocol {
    pub major_type: DeviceType,
    pub sub_type: DeviceSubType,
    /// Total length of the type including the fixed header as u16 in LE order.
    pub length: [u8; 2],
    // followed by payload (dynamically sized)
}

impl DevicePathProtocol {
    pub const GUID: Guid = guid!("09576e91-6d3f-11d2-8e39-00a0c969723b");

    /// Returns the total length of the device path node.
    #[must_use]
    pub const fn length(&self) -> u16 {
        u16::from_le_bytes(self.length)
    }
}

newtype_enum! {
/// Type identifier for a device path node.
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

/// Sub-type identifier for a device path node.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct DeviceSubType(pub u8);

impl DeviceSubType {
    /// PCI Device Path.
    pub const HARDWARE_PCI: Self = Self(1);
    /// PCCARD Device Path.
    pub const HARDWARE_PCCARD: Self = Self(2);
    /// Memory-mapped Device Path.
    pub const HARDWARE_MEMORY_MAPPED: Self = Self(3);
    /// Vendor-Defined Device Path.
    pub const HARDWARE_VENDOR: Self = Self(4);
    /// Controller Device Path.
    pub const HARDWARE_CONTROLLER: Self = Self(5);
    /// BMC Device Path.
    pub const HARDWARE_BMC: Self = Self(6);

    /// ACPI Device Path.
    pub const ACPI: Self = Self(1);
    /// Expanded ACPI Device Path.
    pub const ACPI_EXPANDED: Self = Self(2);
    /// ACPI _ADR Device Path.
    pub const ACPI_ADR: Self = Self(3);
    /// NVDIMM Device Path.
    pub const ACPI_NVDIMM: Self = Self(4);

    /// ATAPI Device Path.
    pub const MESSAGING_ATAPI: Self = Self(1);
    /// SCSI Device Path.
    pub const MESSAGING_SCSI: Self = Self(2);
    /// Fibre Channel Device Path.
    pub const MESSAGING_FIBRE_CHANNEL: Self = Self(3);
    /// 1394 Device Path.
    pub const MESSAGING_1394: Self = Self(4);
    /// USB Device Path.
    pub const MESSAGING_USB: Self = Self(5);
    /// I2O Device Path.
    pub const MESSAGING_I2O: Self = Self(6);
    /// Infiniband Device Path.
    pub const MESSAGING_INFINIBAND: Self = Self(9);
    /// Vendor-Defined Device Path.
    pub const MESSAGING_VENDOR: Self = Self(10);
    /// MAC Address Device Path.
    pub const MESSAGING_MAC_ADDRESS: Self = Self(11);
    /// IPV4 Device Path.
    pub const MESSAGING_IPV4: Self = Self(12);
    /// IPV6 Device Path.
    pub const MESSAGING_IPV6: Self = Self(13);
    /// UART Device Path.
    pub const MESSAGING_UART: Self = Self(14);
    /// USB Class Device Path.
    pub const MESSAGING_USB_CLASS: Self = Self(15);
    /// USB WWID Device Path.
    pub const MESSAGING_USB_WWID: Self = Self(16);
    /// Device Logical Unit.
    pub const MESSAGING_DEVICE_LOGICAL_UNIT: Self = Self(17);
    /// SATA Device Path.
    pub const MESSAGING_SATA: Self = Self(18);
    /// iSCSI Device Path node (base information).
    pub const MESSAGING_ISCSI: Self = Self(19);
    /// VLAN Device Path node.
    pub const MESSAGING_VLAN: Self = Self(20);
    /// Fibre Channel Ex Device Path.
    pub const MESSAGING_FIBRE_CHANNEL_EX: Self = Self(21);
    /// Serial Attached SCSI (SAS) Ex Device Path.
    pub const MESSAGING_SCSI_SAS_EX: Self = Self(22);
    /// NVM Express Namespace Device Path.
    pub const MESSAGING_NVME_NAMESPACE: Self = Self(23);
    /// Uniform Resource Identifiers (URI) Device Path.
    pub const MESSAGING_URI: Self = Self(24);
    /// UFS Device Path.
    pub const MESSAGING_UFS: Self = Self(25);
    /// SD (Secure Digital) Device Path.
    pub const MESSAGING_SD: Self = Self(26);
    /// Bluetooth Device Path.
    pub const MESSAGING_BLUETOOTH: Self = Self(27);
    /// Wi-Fi Device Path.
    pub const MESSAGING_WIFI: Self = Self(28);
    /// eMMC (Embedded Multi-Media Card) Device Path.
    pub const MESSAGING_EMMC: Self = Self(29);
    /// BluetoothLE Device Path.
    pub const MESSAGING_BLUETOOTH_LE: Self = Self(30);
    /// DNS Device Path.
    pub const MESSAGING_DNS: Self = Self(31);
    /// NVDIMM Namespace Device Path.
    pub const MESSAGING_NVDIMM_NAMESPACE: Self = Self(32);
    /// REST Service Device Path.
    pub const MESSAGING_REST_SERVICE: Self = Self(33);
    /// NVME over Fabric (NVMe-oF) Namespace Device Path.
    pub const MESSAGING_NVME_OF_NAMESPACE: Self = Self(34);

    /// Hard Drive Media Device Path.
    pub const MEDIA_HARD_DRIVE: Self = Self(1);
    /// CD-ROM Media Device Path.
    pub const MEDIA_CD_ROM: Self = Self(2);
    /// Vendor-Defined Media Device Path.
    pub const MEDIA_VENDOR: Self = Self(3);
    /// File Path Media Device Path.
    pub const MEDIA_FILE_PATH: Self = Self(4);
    /// Media Protocol Device Path.
    pub const MEDIA_PROTOCOL: Self = Self(5);
    /// PIWG Firmware File.
    pub const MEDIA_PIWG_FIRMWARE_FILE: Self = Self(6);
    /// PIWG Firmware Volume.
    pub const MEDIA_PIWG_FIRMWARE_VOLUME: Self = Self(7);
    /// Relative Offset Range.
    pub const MEDIA_RELATIVE_OFFSET_RANGE: Self = Self(8);
    /// RAM Disk Device Path.
    pub const MEDIA_RAM_DISK: Self = Self(9);

    /// BIOS Boot Specification Device Path.
    pub const BIOS_BOOT_SPECIFICATION: Self = Self(1);

    /// End this instance of a Device Path and start a new one.
    pub const END_INSTANCE: Self = Self(0x01);
    /// End entire Device Path.
    pub const END_ENTIRE: Self = Self(0xff);
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

#[derive(Debug)]
#[repr(C)]
pub struct DevicePathUtilitiesProtocol {
    pub get_device_path_size:
        unsafe extern "efiapi" fn(device_path: *const DevicePathProtocol) -> usize,
    pub duplicate_device_path: unsafe extern "efiapi" fn(
        device_path: *const DevicePathProtocol,
    ) -> *const DevicePathProtocol,
    pub append_device_path: unsafe extern "efiapi" fn(
        src1: *const DevicePathProtocol,
        src2: *const DevicePathProtocol,
    ) -> *const DevicePathProtocol,
    pub append_device_node: unsafe extern "efiapi" fn(
        device_path: *const DevicePathProtocol,
        device_node: *const DevicePathProtocol,
    ) -> *const DevicePathProtocol,
    pub append_device_path_instance: unsafe extern "efiapi" fn(
        device_path: *const DevicePathProtocol,
        device_path_instance: *const DevicePathProtocol,
    ) -> *const DevicePathProtocol,
    pub get_next_device_path_instance: unsafe extern "efiapi" fn(
        device_path_instance: *mut *const DevicePathProtocol,
        device_path_instance_size: *mut usize,
    ) -> *const DevicePathProtocol,
    pub is_device_path_multi_instance:
        unsafe extern "efiapi" fn(device_path: *const DevicePathProtocol) -> bool,
    pub create_device_node: unsafe extern "efiapi" fn(
        node_type: DeviceType,
        node_sub_type: DeviceSubType,
        node_length: u16,
    ) -> *const DevicePathProtocol,
}

impl DevicePathUtilitiesProtocol {
    pub const GUID: Guid = guid!("0379be4e-d706-437d-b037-edb82fb772a4");
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem;

    /// Test that ensures the struct is packed. Thus, we don't need to
    /// explicitly specify `packed`.
    #[test]
    fn abi() {
        assert_eq!(mem::size_of::<DevicePathProtocol>(), 4);
        assert_eq!(mem::align_of::<DevicePathProtocol>(), 1);
    }
}
