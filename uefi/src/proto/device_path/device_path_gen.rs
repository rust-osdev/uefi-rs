// DO NOT EDIT
//
// This file was automatically generated with:
// `cargo xtask gen-code`
//
// See //xtask/src/device_path/README.md for more details.

use crate::data_types::UnalignedSlice;
use crate::proto::device_path::{
    DevicePathHeader, DevicePathNode, DeviceSubType, DeviceType, NodeConversionError,
};
use crate::proto::network::IpAddress;
use crate::table::boot::MemoryType;
use crate::{guid, Guid};
use bitflags::bitflags;
use core::mem::{size_of, size_of_val};
use core::ptr::{self, addr_of};
use core::{fmt, slice};
/// Device path nodes for [`DeviceType::END`].
pub mod end {
    use super::*;
    /// Node that terminates a [`DevicePathInstance`].
    ///
    /// [`DevicePathInstance`]: crate::proto::device_path::DevicePathInstance
    #[repr(C, packed)]
    pub struct Instance {
        pub(super) header: DevicePathHeader,
    }

    impl Instance {}

    impl fmt::Debug for Instance {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Instance").finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Instance {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Instance>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Instance = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// Node that terminates an entire [`DevicePath`].
    ///
    /// [`DevicePath`]: crate::proto::device_path::DevicePath
    #[repr(C, packed)]
    pub struct Entire {
        pub(super) header: DevicePathHeader,
    }

    impl Entire {}

    impl fmt::Debug for Entire {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Entire").finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Entire {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Entire>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Entire = node.cast();
            Ok(unsafe { &*node })
        }
    }
}

/// Device path nodes for [`DeviceType::HARDWARE`].
pub mod hardware {
    use super::*;
    /// PCI hardware device path node.
    #[repr(C, packed)]
    pub struct Pci {
        pub(super) header: DevicePathHeader,
        pub(super) function: u8,
        pub(super) device: u8,
    }

    impl Pci {
        /// PCI function number.
        pub fn function(&self) -> u8 {
            self.function
        }

        /// PCI device number.
        pub fn device(&self) -> u8 {
            self.device
        }
    }

    impl fmt::Debug for Pci {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Pci")
                .field("function", &{ self.function })
                .field("device", &{ self.device })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Pci {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Pci>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Pci = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// PCCARD hardware device path node.
    #[repr(C, packed)]
    pub struct Pccard {
        pub(super) header: DevicePathHeader,
        pub(super) function: u8,
    }

    impl Pccard {
        /// Function number starting from 0.
        pub fn function(&self) -> u8 {
            self.function
        }
    }

    impl fmt::Debug for Pccard {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Pccard")
                .field("function", &{ self.function })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Pccard {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Pccard>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Pccard = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// Memory mapped hardware device path node.
    #[repr(C, packed)]
    pub struct MemoryMapped {
        pub(super) header: DevicePathHeader,
        pub(super) memory_type: MemoryType,
        pub(super) start_address: u64,
        pub(super) end_address: u64,
    }

    impl MemoryMapped {
        /// Memory type.
        pub fn memory_type(&self) -> MemoryType {
            self.memory_type
        }

        /// Starting memory address.
        pub fn start_address(&self) -> u64 {
            self.start_address
        }

        /// Ending memory address.
        pub fn end_address(&self) -> u64 {
            self.end_address
        }
    }

    impl fmt::Debug for MemoryMapped {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("MemoryMapped")
                .field("memory_type", &{ self.memory_type })
                .field("start_address", &{ self.start_address })
                .field("end_address", &{ self.end_address })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &MemoryMapped {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<MemoryMapped>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const MemoryMapped = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// Vendor-defined hardware device path node.
    #[repr(C, packed)]
    pub struct Vendor {
        pub(super) header: DevicePathHeader,
        pub(super) vendor_guid: Guid,
        pub(super) vendor_defined_data: [u8],
    }

    impl Vendor {
        /// Vendor-assigned GUID that defines the data that follows.
        pub fn vendor_guid(&self) -> Guid {
            self.vendor_guid
        }

        /// Vendor-defined data.
        pub fn vendor_defined_data(&self) -> &[u8] {
            &self.vendor_defined_data
        }
    }

    impl fmt::Debug for Vendor {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Vendor")
                .field("vendor_guid", &{ self.vendor_guid })
                .field("vendor_defined_data", {
                    let ptr = addr_of!(self.vendor_defined_data);
                    let (ptr, len) = ptr.to_raw_parts();
                    let byte_len = size_of::<u8>() * len;
                    unsafe { &slice::from_raw_parts(ptr.cast::<u8>(), byte_len) }
                })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Vendor {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            let static_size = 20usize;
            let dst_size = size_of_val(node)
                .checked_sub(static_size)
                .ok_or(NodeConversionError::InvalidLength)?;
            let elem_size = size_of::<u8>();
            if dst_size % elem_size != 0 {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Vendor = ptr::from_raw_parts(node.cast(), dst_size / elem_size);
            Ok(unsafe { &*node })
        }
    }

    /// Controller hardware device path node.
    #[repr(C, packed)]
    pub struct Controller {
        pub(super) header: DevicePathHeader,
        pub(super) controller_number: u32,
    }

    impl Controller {
        /// Controller number.
        pub fn controller_number(&self) -> u32 {
            self.controller_number
        }
    }

    impl fmt::Debug for Controller {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Controller")
                .field("controller_number", &{ self.controller_number })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Controller {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Controller>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Controller = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// Baseboard Management Controller (BMC) host interface hardware
    /// device path node.
    #[repr(C, packed)]
    pub struct Bmc {
        pub(super) header: DevicePathHeader,
        pub(super) interface_type: crate::proto::device_path::hardware::BmcInterfaceType,
        pub(super) base_address: u64,
    }

    impl Bmc {
        /// Host interface type.
        pub fn interface_type(&self) -> crate::proto::device_path::hardware::BmcInterfaceType {
            self.interface_type
        }

        /// Base address of the BMC. If the least-significant bit of the
        /// field is a 1 then the address is in I/O space, otherwise the
        /// address is memory-mapped.
        pub fn base_address(&self) -> u64 {
            self.base_address
        }
    }

    impl fmt::Debug for Bmc {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Bmc")
                .field("interface_type", &{ self.interface_type })
                .field("base_address", &{ self.base_address })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Bmc {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Bmc>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Bmc = node.cast();
            Ok(unsafe { &*node })
        }
    }

    newtype_enum! { # [doc = " Baseboard Management Controller (BMC) host interface type."] pub enum BmcInterfaceType : u8 => { # [doc = " Unknown."] UNKNOWN = 0x00 , # [doc = " Keyboard controller style."] KEYBOARD_CONTROLLER_STYLE = 0x01 , # [doc = " Server management interface chip."] SERVER_MANAGEMENT_INTERFACE_CHIP = 0x02 , # [doc = " Block transfer."] BLOCK_TRANSFER = 0x03 , }

    }
}

/// Device path nodes for [`DeviceType::ACPI`].
pub mod acpi {
    use super::*;
    /// ACPI device path node.
    #[repr(C, packed)]
    pub struct Acpi {
        pub(super) header: DevicePathHeader,
        pub(super) hid: u32,
        pub(super) uid: u32,
    }

    impl Acpi {
        /// Device's PnP hardware ID stored in a numeric 32-bit
        /// compressed EISA-type ID.
        pub fn hid(&self) -> u32 {
            self.hid
        }

        /// Unique ID that is required by ACPI if two devices have the
        /// same HID.
        pub fn uid(&self) -> u32 {
            self.uid
        }
    }

    impl fmt::Debug for Acpi {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Acpi")
                .field("hid", &{ self.hid })
                .field("uid", &{ self.uid })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Acpi {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Acpi>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Acpi = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// Expanded ACPI device path node.
    #[repr(C, packed)]
    pub struct Expanded {
        pub(super) header: DevicePathHeader,
        pub(super) hid: u32,
        pub(super) uid: u32,
        pub(super) cid: u32,
        pub(super) data: [u8],
    }

    impl Expanded {
        /// Device's PnP hardware ID stored in a numeric 32-bit compressed
        /// EISA-type ID.
        pub fn hid(&self) -> u32 {
            self.hid
        }

        /// Unique ID that is required by ACPI if two devices have the
        /// same HID.
        pub fn uid(&self) -> u32 {
            self.uid
        }

        /// Device's compatible PnP hardware ID stored in a numeric 32-bit
        /// compressed EISA-type ID.
        pub fn cid(&self) -> u32 {
            self.cid
        }

        /// Device's PnP hardware ID stored as a null-terminated ASCII
        /// string. This value must match the corresponding HID in the
        /// ACPI name space. If the length of this string not including
        /// the null-terminator is 0, then the numeric HID is used.
        pub fn hid_str(&self) -> &[u8] {
            self.get_hid_str()
        }

        /// Unique ID that is required by ACPI if two devices have the
        /// same HID. This value is stored as a null-terminated ASCII
        /// string. If the length of this string not including the
        /// null-terminator is 0, then the numeric UID is used.
        pub fn uid_str(&self) -> &[u8] {
            self.get_uid_str()
        }

        /// Device's compatible PnP hardware ID stored as a
        /// null-terminated ASCII string. If the length of this string
        /// not including the null-terminator is 0, then the numeric CID
        /// is used.
        pub fn cid_str(&self) -> &[u8] {
            self.get_cid_str()
        }
    }

    impl fmt::Debug for Expanded {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Expanded")
                .field("hid", &{ self.hid })
                .field("uid", &{ self.uid })
                .field("cid", &{ self.cid })
                .field("data", &&self.data)
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Expanded {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            let static_size = 16usize;
            let dst_size = size_of_val(node)
                .checked_sub(static_size)
                .ok_or(NodeConversionError::InvalidLength)?;
            let elem_size = size_of::<u8>();
            if dst_size % elem_size != 0 {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Expanded = ptr::from_raw_parts(node.cast(), dst_size / elem_size);
            Ok(unsafe { &*node })
        }
    }

    /// ADR ACPI device path node.
    #[repr(C, packed)]
    pub struct Adr {
        pub(super) header: DevicePathHeader,
        pub(super) adr: [u32],
    }

    impl Adr {
        /// ADR values. For video output devices the value of this field
        /// comes from Table B-2 ACPI 3.0 specification. At least one
        /// ADR value is required.
        pub fn adr(&self) -> UnalignedSlice<u32> {
            let ptr: *const [u32] = addr_of!(self.adr);
            let (ptr, len): (*const (), usize) = ptr.to_raw_parts();
            unsafe { UnalignedSlice::new(ptr.cast::<u32>(), len) }
        }
    }

    impl fmt::Debug for Adr {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Adr")
                .field("adr", {
                    let ptr = addr_of!(self.adr);
                    let (ptr, len) = ptr.to_raw_parts();
                    let byte_len = size_of::<u32>() * len;
                    unsafe { &slice::from_raw_parts(ptr.cast::<u8>(), byte_len) }
                })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Adr {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            let static_size = 4usize;
            let dst_size = size_of_val(node)
                .checked_sub(static_size)
                .ok_or(NodeConversionError::InvalidLength)?;
            let elem_size = size_of::<u32>();
            if dst_size % elem_size != 0 {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Adr = ptr::from_raw_parts(node.cast(), dst_size / elem_size);
            Ok(unsafe { &*node })
        }
    }

    /// NVDIMM ACPI device path node.
    #[repr(C, packed)]
    pub struct Nvdimm {
        pub(super) header: DevicePathHeader,
        pub(super) nfit_device_handle: u32,
    }

    impl Nvdimm {
        /// NFIT device handle.
        pub fn nfit_device_handle(&self) -> u32 {
            self.nfit_device_handle
        }
    }

    impl fmt::Debug for Nvdimm {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Nvdimm")
                .field("nfit_device_handle", &{ self.nfit_device_handle })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Nvdimm {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Nvdimm>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Nvdimm = node.cast();
            Ok(unsafe { &*node })
        }
    }

    impl Expanded {
        fn get_hid_str(&self) -> &[u8] {
            get_acpi_expanded_substr(&self.data, 0)
        }

        fn get_uid_str(&self) -> &[u8] {
            get_acpi_expanded_substr(&self.data, 1)
        }

        fn get_cid_str(&self) -> &[u8] {
            get_acpi_expanded_substr(&self.data, 2)
        }
    }

    /// Get the indices of the three nulls in the combined hid/uid/cid
    /// string. This never fails; if some nulls are missing then `None`
    /// is returned for those indices. If more than three nulls are
    /// present then the extra ones are ignored.
    fn acpi_expanded_null_indices(data: &[u8]) -> [Option<usize>; 3] {
        let mut iter = data
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(index, byte)| if byte == 0 { Some(index) } else { None })
            .fuse();
        [iter.next(), iter.next(), iter.next()]
    }

    /// Get the hid, uid, or cid string from the combined string. The
    /// returned string includes the trailing null if possible; if the
    /// substring was not properly null terminated then it ends at the
    /// end of `data`.
    ///
    /// This never fails; if there aren't enough nulls in the input
    /// string then an empty slice may be returned.
    fn get_acpi_expanded_substr(data: &[u8], string_index: usize) -> &[u8] {
        let [n0, n1, n2] = acpi_expanded_null_indices(data);
        let mut start = data.len();
        let mut end = start;
        match string_index {
            0 => {
                start = 0;
                if let Some(n0) = n0 {
                    end = n0 + 1;
                }
            }

            1 => {
                if let Some(n0) = n0 {
                    start = n0 + 1;
                    if let Some(n1) = n1 {
                        end = n1 + 1;
                    }
                }
            }

            2 => {
                if let Some(n1) = n1 {
                    start = n1 + 1;
                    if let Some(n2) = n2 {
                        end = n2 + 1;
                    }
                }
            }

            _ => {
                unreachable!("invalid string index")
            }
        }

        data.get(start..end).unwrap_or(&[])
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_get_acpi_expanded_substr() {
            let s = b"ab\0cd\0ef\0";
            assert_eq!(get_acpi_expanded_substr(s, 0), b"ab\0");
            assert_eq!(get_acpi_expanded_substr(s, 1), b"cd\0");
            assert_eq!(get_acpi_expanded_substr(s, 2), b"ef\0");
            let s = b"\0\0\0";
            assert_eq!(get_acpi_expanded_substr(s, 0), b"\0");
            assert_eq!(get_acpi_expanded_substr(s, 1), b"\0");
            assert_eq!(get_acpi_expanded_substr(s, 2), b"\0");
            let s = b"ab\0cd\0";
            assert_eq!(get_acpi_expanded_substr(s, 0), b"ab\0");
            assert_eq!(get_acpi_expanded_substr(s, 1), b"cd\0");
            assert_eq!(get_acpi_expanded_substr(s, 2), b"");
            let s = b"ab\0";
            assert_eq!(get_acpi_expanded_substr(s, 0), b"ab\0");
            assert_eq!(get_acpi_expanded_substr(s, 1), b"");
            assert_eq!(get_acpi_expanded_substr(s, 2), b"");
            let s = b"ab\0cd\0ef";
            assert_eq!(get_acpi_expanded_substr(s, 0), b"ab\0");
            assert_eq!(get_acpi_expanded_substr(s, 1), b"cd\0");
            assert_eq!(get_acpi_expanded_substr(s, 2), b"ef");
            let s = b"ab\0cd";
            assert_eq!(get_acpi_expanded_substr(s, 0), b"ab\0");
            assert_eq!(get_acpi_expanded_substr(s, 1), b"cd");
            assert_eq!(get_acpi_expanded_substr(s, 2), b"");
            let s = b"ab";
            assert_eq!(get_acpi_expanded_substr(s, 0), b"ab");
            assert_eq!(get_acpi_expanded_substr(s, 1), b"");
            assert_eq!(get_acpi_expanded_substr(s, 2), b"");
            let s = b"";
            assert_eq!(get_acpi_expanded_substr(s, 0), b"");
            assert_eq!(get_acpi_expanded_substr(s, 1), b"");
            assert_eq!(get_acpi_expanded_substr(s, 2), b"");
        }
    }
}

/// Device path nodes for [`DeviceType::MESSAGING`].
pub mod messaging {
    use super::*;
    /// ATAPI messaging device path node.
    #[repr(C, packed)]
    pub struct Atapi {
        pub(super) header: DevicePathHeader,
        pub(super) primary_secondary: crate::proto::device_path::messaging::PrimarySecondary,
        pub(super) master_slave: crate::proto::device_path::messaging::MasterSlave,
        pub(super) logical_unit_number: u16,
    }

    impl Atapi {
        /// Whether the ATAPI device is primary or secondary.
        pub fn primary_secondary(&self) -> crate::proto::device_path::messaging::PrimarySecondary {
            self.primary_secondary
        }

        /// Whether the ATAPI device is master or slave.
        pub fn master_slave(&self) -> crate::proto::device_path::messaging::MasterSlave {
            self.master_slave
        }

        /// Logical Unit Number (LUN).
        pub fn logical_unit_number(&self) -> u16 {
            self.logical_unit_number
        }
    }

    impl fmt::Debug for Atapi {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Atapi")
                .field("primary_secondary", &{ self.primary_secondary })
                .field("master_slave", &{ self.master_slave })
                .field("logical_unit_number", &{ self.logical_unit_number })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Atapi {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Atapi>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Atapi = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// SCSI messaging device path node.
    #[repr(C, packed)]
    pub struct Scsi {
        pub(super) header: DevicePathHeader,
        pub(super) target_id: u16,
        pub(super) logical_unit_number: u16,
    }

    impl Scsi {
        /// Target ID on the SCSI bus.
        pub fn target_id(&self) -> u16 {
            self.target_id
        }

        /// Logical Unit Number.
        pub fn logical_unit_number(&self) -> u16 {
            self.logical_unit_number
        }
    }

    impl fmt::Debug for Scsi {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Scsi")
                .field("target_id", &{ self.target_id })
                .field("logical_unit_number", &{ self.logical_unit_number })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Scsi {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Scsi>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Scsi = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// Fibre channel messaging device path node.
    #[repr(C, packed)]
    pub struct FibreChannel {
        pub(super) header: DevicePathHeader,
        pub(super) _reserved: u32,
        pub(super) world_wide_name: u64,
        pub(super) logical_unit_number: u64,
    }

    impl FibreChannel {
        /// Fibre Channel World Wide Name.
        pub fn world_wide_name(&self) -> u64 {
            self.world_wide_name
        }

        /// Fibre Channel Logical Unit Number.
        pub fn logical_unit_number(&self) -> u64 {
            self.logical_unit_number
        }
    }

    impl fmt::Debug for FibreChannel {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("FibreChannel")
                .field("_reserved", &{ self._reserved })
                .field("world_wide_name", &{ self.world_wide_name })
                .field("logical_unit_number", &{ self.logical_unit_number })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &FibreChannel {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<FibreChannel>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const FibreChannel = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// Fibre channel extended messaging device path node.
    #[repr(C, packed)]
    pub struct FibreChannelEx {
        pub(super) header: DevicePathHeader,
        pub(super) _reserved: u32,
        pub(super) world_wide_name: [u8; 8usize],
        pub(super) logical_unit_number: [u8; 8usize],
    }

    impl FibreChannelEx {
        /// Fibre Channel end device port name (aka World Wide Name).
        pub fn world_wide_name(&self) -> [u8; 8usize] {
            self.world_wide_name
        }

        /// Fibre Channel Logical Unit Number.
        pub fn logical_unit_number(&self) -> [u8; 8usize] {
            self.logical_unit_number
        }
    }

    impl fmt::Debug for FibreChannelEx {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("FibreChannelEx")
                .field("_reserved", &{ self._reserved })
                .field("world_wide_name", &{ self.world_wide_name })
                .field("logical_unit_number", &{ self.logical_unit_number })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &FibreChannelEx {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<FibreChannelEx>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const FibreChannelEx = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// 1394 messaging device path node.
    #[repr(C, packed)]
    pub struct Ieee1394 {
        pub(super) header: DevicePathHeader,
        pub(super) _reserved: u32,
        pub(super) guid: [u8; 8usize],
    }

    impl Ieee1394 {
        /// 1394 Global Unique ID. Note that this is not the same as a
        /// UEFI GUID.
        pub fn guid(&self) -> [u8; 8usize] {
            self.guid
        }
    }

    impl fmt::Debug for Ieee1394 {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Ieee1394")
                .field("_reserved", &{ self._reserved })
                .field("guid", &{ self.guid })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Ieee1394 {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Ieee1394>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Ieee1394 = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// USB messaging device path node.
    #[repr(C, packed)]
    pub struct Usb {
        pub(super) header: DevicePathHeader,
        pub(super) parent_port_number: u8,
        pub(super) interface: u8,
    }

    impl Usb {
        /// USB parent port number.
        pub fn parent_port_number(&self) -> u8 {
            self.parent_port_number
        }

        /// USB interface number.
        pub fn interface(&self) -> u8 {
            self.interface
        }
    }

    impl fmt::Debug for Usb {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Usb")
                .field("parent_port_number", &{ self.parent_port_number })
                .field("interface", &{ self.interface })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Usb {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Usb>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Usb = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// SATA messaging device path node.
    #[repr(C, packed)]
    pub struct Sata {
        pub(super) header: DevicePathHeader,
        pub(super) hba_port_number: u16,
        pub(super) port_multiplier_port_number: u16,
        pub(super) logical_unit_number: u16,
    }

    impl Sata {
        /// The HBA port number that facilitates the connection to the
        /// device or a port multiplier. The value 0xffff is reserved.
        pub fn hba_port_number(&self) -> u16 {
            self.hba_port_number
        }

        /// the port multiplier port number that facilitates the
        /// connection to the device. Must be set to 0xffff if the
        /// device is directly connected to the HBA.
        pub fn port_multiplier_port_number(&self) -> u16 {
            self.port_multiplier_port_number
        }

        /// Logical unit number.
        pub fn logical_unit_number(&self) -> u16 {
            self.logical_unit_number
        }
    }

    impl fmt::Debug for Sata {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Sata")
                .field("hba_port_number", &{ self.hba_port_number })
                .field("port_multiplier_port_number", &{
                    self.port_multiplier_port_number
                })
                .field("logical_unit_number", &{ self.logical_unit_number })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Sata {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Sata>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Sata = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// USB World Wide ID (WWID) messaging device path node.
    #[repr(C, packed)]
    pub struct UsbWwid {
        pub(super) header: DevicePathHeader,
        pub(super) interface_number: u16,
        pub(super) device_vendor_id: u16,
        pub(super) device_product_id: u16,
        pub(super) serial_number: [u16],
    }

    impl UsbWwid {
        /// USB interface number.
        pub fn interface_number(&self) -> u16 {
            self.interface_number
        }

        /// USB vendor ID.
        pub fn device_vendor_id(&self) -> u16 {
            self.device_vendor_id
        }

        /// USB product ID.
        pub fn device_product_id(&self) -> u16 {
            self.device_product_id
        }

        /// Last 64 (or fewer) characters of the USB Serial number.
        pub fn serial_number(&self) -> UnalignedSlice<u16> {
            let ptr: *const [u16] = addr_of!(self.serial_number);
            let (ptr, len): (*const (), usize) = ptr.to_raw_parts();
            unsafe { UnalignedSlice::new(ptr.cast::<u16>(), len) }
        }
    }

    impl fmt::Debug for UsbWwid {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("UsbWwid")
                .field("interface_number", &{ self.interface_number })
                .field("device_vendor_id", &{ self.device_vendor_id })
                .field("device_product_id", &{ self.device_product_id })
                .field("serial_number", {
                    let ptr = addr_of!(self.serial_number);
                    let (ptr, len) = ptr.to_raw_parts();
                    let byte_len = size_of::<u16>() * len;
                    unsafe { &slice::from_raw_parts(ptr.cast::<u8>(), byte_len) }
                })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &UsbWwid {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            let static_size = 10usize;
            let dst_size = size_of_val(node)
                .checked_sub(static_size)
                .ok_or(NodeConversionError::InvalidLength)?;
            let elem_size = size_of::<u16>();
            if dst_size % elem_size != 0 {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const UsbWwid = ptr::from_raw_parts(node.cast(), dst_size / elem_size);
            Ok(unsafe { &*node })
        }
    }

    /// Device logical unit messaging device path node.
    #[repr(C, packed)]
    pub struct DeviceLogicalUnit {
        pub(super) header: DevicePathHeader,
        pub(super) logical_unit_number: u8,
    }

    impl DeviceLogicalUnit {
        /// Logical Unit Number.
        pub fn logical_unit_number(&self) -> u8 {
            self.logical_unit_number
        }
    }

    impl fmt::Debug for DeviceLogicalUnit {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("DeviceLogicalUnit")
                .field("logical_unit_number", &{ self.logical_unit_number })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &DeviceLogicalUnit {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<DeviceLogicalUnit>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const DeviceLogicalUnit = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// USB class messaging device path node.
    #[repr(C, packed)]
    pub struct UsbClass {
        pub(super) header: DevicePathHeader,
        pub(super) vendor_id: u16,
        pub(super) product_id: u16,
        pub(super) device_class: u8,
        pub(super) device_subclass: u8,
        pub(super) device_protocol: u8,
    }

    impl UsbClass {
        /// USB vendor ID.
        pub fn vendor_id(&self) -> u16 {
            self.vendor_id
        }

        /// USB product ID.
        pub fn product_id(&self) -> u16 {
            self.product_id
        }

        /// USB device class.
        pub fn device_class(&self) -> u8 {
            self.device_class
        }

        /// USB device subclass.
        pub fn device_subclass(&self) -> u8 {
            self.device_subclass
        }

        /// USB device protocol.
        pub fn device_protocol(&self) -> u8 {
            self.device_protocol
        }
    }

    impl fmt::Debug for UsbClass {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("UsbClass")
                .field("vendor_id", &{ self.vendor_id })
                .field("product_id", &{ self.product_id })
                .field("device_class", &{ self.device_class })
                .field("device_subclass", &{ self.device_subclass })
                .field("device_protocol", &{ self.device_protocol })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &UsbClass {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<UsbClass>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const UsbClass = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// I2O messaging device path node.
    #[repr(C, packed)]
    pub struct I2o {
        pub(super) header: DevicePathHeader,
        pub(super) target_id: u32,
    }

    impl I2o {
        /// Target ID (TID).
        pub fn target_id(&self) -> u32 {
            self.target_id
        }
    }

    impl fmt::Debug for I2o {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("I2o")
                .field("target_id", &{ self.target_id })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &I2o {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<I2o>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const I2o = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// MAC address messaging device path node.
    #[repr(C, packed)]
    pub struct MacAddress {
        pub(super) header: DevicePathHeader,
        pub(super) mac_address: [u8; 32usize],
        pub(super) interface_type: u8,
    }

    impl MacAddress {
        /// MAC address for a network interface, padded with zeros.
        pub fn mac_address(&self) -> [u8; 32usize] {
            self.mac_address
        }

        /// Network interface type. See
        /// <https://www.iana.org/assignments/smi-numbers/smi-numbers.xhtml#smi-numbers-5>
        pub fn interface_type(&self) -> u8 {
            self.interface_type
        }
    }

    impl fmt::Debug for MacAddress {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("MacAddress")
                .field("mac_address", &{ self.mac_address })
                .field("interface_type", &{ self.interface_type })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &MacAddress {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<MacAddress>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const MacAddress = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// IPv4 messaging device path node.
    #[repr(C, packed)]
    pub struct Ipv4 {
        pub(super) header: DevicePathHeader,
        pub(super) local_ip_address: [u8; 4usize],
        pub(super) remote_ip_address: [u8; 4usize],
        pub(super) local_port: u16,
        pub(super) remote_port: u16,
        pub(super) protocol: u16,
        pub(super) ip_address_origin: crate::proto::device_path::messaging::Ipv4AddressOrigin,
        pub(super) gateway_ip_address: [u8; 4usize],
        pub(super) subnet_mask: [u8; 4usize],
    }

    impl Ipv4 {
        /// Local IPv4 address.
        pub fn local_ip_address(&self) -> [u8; 4usize] {
            self.local_ip_address
        }

        /// Remote IPv4 address.
        pub fn remote_ip_address(&self) -> [u8; 4usize] {
            self.remote_ip_address
        }

        /// Local port number.
        pub fn local_port(&self) -> u16 {
            self.local_port
        }

        /// Remote port number.
        pub fn remote_port(&self) -> u16 {
            self.remote_port
        }

        /// Network protocol. See
        /// <https://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml>
        pub fn protocol(&self) -> u16 {
            self.protocol
        }

        /// Whether the source IP address is static or assigned via DHCP.
        pub fn ip_address_origin(&self) -> crate::proto::device_path::messaging::Ipv4AddressOrigin {
            self.ip_address_origin
        }

        /// Gateway IP address.
        pub fn gateway_ip_address(&self) -> [u8; 4usize] {
            self.gateway_ip_address
        }

        /// Subnet mask.
        pub fn subnet_mask(&self) -> [u8; 4usize] {
            self.subnet_mask
        }
    }

    impl fmt::Debug for Ipv4 {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Ipv4")
                .field("local_ip_address", &{ self.local_ip_address })
                .field("remote_ip_address", &{ self.remote_ip_address })
                .field("local_port", &{ self.local_port })
                .field("remote_port", &{ self.remote_port })
                .field("protocol", &{ self.protocol })
                .field("ip_address_origin", &{ self.ip_address_origin })
                .field("gateway_ip_address", &{ self.gateway_ip_address })
                .field("subnet_mask", &{ self.subnet_mask })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Ipv4 {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Ipv4>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Ipv4 = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// IPv6 messaging device path node.
    #[repr(C, packed)]
    pub struct Ipv6 {
        pub(super) header: DevicePathHeader,
        pub(super) local_ip_address: [u8; 16usize],
        pub(super) remote_ip_address: [u8; 16usize],
        pub(super) local_port: u16,
        pub(super) remote_port: u16,
        pub(super) protocol: u16,
        pub(super) ip_address_origin: crate::proto::device_path::messaging::Ipv6AddressOrigin,
        pub(super) prefix_length: u8,
        pub(super) gateway_ip_address: [u8; 16usize],
    }

    impl Ipv6 {
        /// Local Ipv6 address.
        pub fn local_ip_address(&self) -> [u8; 16usize] {
            self.local_ip_address
        }

        /// Remote Ipv6 address.
        pub fn remote_ip_address(&self) -> [u8; 16usize] {
            self.remote_ip_address
        }

        /// Local port number.
        pub fn local_port(&self) -> u16 {
            self.local_port
        }

        /// Remote port number.
        pub fn remote_port(&self) -> u16 {
            self.remote_port
        }

        /// Network protocol. See
        /// <https://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml>
        pub fn protocol(&self) -> u16 {
            self.protocol
        }

        /// Origin of the local IP address.
        pub fn ip_address_origin(&self) -> crate::proto::device_path::messaging::Ipv6AddressOrigin {
            self.ip_address_origin
        }

        /// Prefix length.
        pub fn prefix_length(&self) -> u8 {
            self.prefix_length
        }

        /// Gateway IP address.
        pub fn gateway_ip_address(&self) -> [u8; 16usize] {
            self.gateway_ip_address
        }
    }

    impl fmt::Debug for Ipv6 {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Ipv6")
                .field("local_ip_address", &{ self.local_ip_address })
                .field("remote_ip_address", &{ self.remote_ip_address })
                .field("local_port", &{ self.local_port })
                .field("remote_port", &{ self.remote_port })
                .field("protocol", &{ self.protocol })
                .field("ip_address_origin", &{ self.ip_address_origin })
                .field("prefix_length", &{ self.prefix_length })
                .field("gateway_ip_address", &{ self.gateway_ip_address })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Ipv6 {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Ipv6>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Ipv6 = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// VLAN messaging device path node.
    #[repr(C, packed)]
    pub struct Vlan {
        pub(super) header: DevicePathHeader,
        pub(super) vlan_id: u16,
    }

    impl Vlan {
        /// VLAN identifier (0-4094).
        pub fn vlan_id(&self) -> u16 {
            self.vlan_id
        }
    }

    impl fmt::Debug for Vlan {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Vlan")
                .field("vlan_id", &{ self.vlan_id })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Vlan {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Vlan>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Vlan = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// InfiniBand messaging device path node.
    #[repr(C, packed)]
    pub struct Infiniband {
        pub(super) header: DevicePathHeader,
        pub(super) resource_flags: crate::proto::device_path::messaging::InfinibandResourceFlags,
        pub(super) port_gid: [u8; 16usize],
        pub(super) ioc_guid_or_service_id: u64,
        pub(super) target_port_id: u64,
        pub(super) device_id: u64,
    }

    impl Infiniband {
        /// Flags to identify/manage InfiniBand elements.
        pub fn resource_flags(
            &self,
        ) -> crate::proto::device_path::messaging::InfinibandResourceFlags {
            self.resource_flags
        }

        /// 128-bit Global Identifier for remote fabric port. Note that
        /// this is not the same as a UEFI GUID.
        pub fn port_gid(&self) -> [u8; 16usize] {
            self.port_gid
        }

        /// IOC GUID if bit 0 of `resource_flags` is unset, or Service
        /// ID if bit 0 of `resource_flags` is set.
        pub fn ioc_guid_or_service_id(&self) -> u64 {
            self.ioc_guid_or_service_id
        }

        /// 64-bit persistent ID of remote IOC port.
        pub fn target_port_id(&self) -> u64 {
            self.target_port_id
        }

        /// 64-bit persistent ID of remote device..
        pub fn device_id(&self) -> u64 {
            self.device_id
        }
    }

    impl fmt::Debug for Infiniband {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Infiniband")
                .field("resource_flags", &{ self.resource_flags })
                .field("port_gid", &{ self.port_gid })
                .field("ioc_guid_or_service_id", &{ self.ioc_guid_or_service_id })
                .field("target_port_id", &{ self.target_port_id })
                .field("device_id", &{ self.device_id })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Infiniband {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Infiniband>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Infiniband = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// UART messaging device path node.
    #[repr(C, packed)]
    pub struct Uart {
        pub(super) header: DevicePathHeader,
        pub(super) _reserved: u32,
        pub(super) baud_rate: u64,
        pub(super) data_bits: u8,
        pub(super) parity: crate::proto::device_path::messaging::Parity,
        pub(super) stop_bits: crate::proto::device_path::messaging::StopBits,
    }

    impl Uart {
        /// Baud rate setting, or 0 to use the device's default.
        pub fn baud_rate(&self) -> u64 {
            self.baud_rate
        }

        /// Number of data bits, or 0 to use the device's default.
        pub fn data_bits(&self) -> u8 {
            self.data_bits
        }

        /// Parity setting.
        pub fn parity(&self) -> crate::proto::device_path::messaging::Parity {
            self.parity
        }

        /// Number of stop bits.
        pub fn stop_bits(&self) -> crate::proto::device_path::messaging::StopBits {
            self.stop_bits
        }
    }

    impl fmt::Debug for Uart {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Uart")
                .field("_reserved", &{ self._reserved })
                .field("baud_rate", &{ self.baud_rate })
                .field("data_bits", &{ self.data_bits })
                .field("parity", &{ self.parity })
                .field("stop_bits", &{ self.stop_bits })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Uart {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Uart>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Uart = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// Vendor-defined messaging device path node.
    #[repr(C, packed)]
    pub struct Vendor {
        pub(super) header: DevicePathHeader,
        pub(super) vendor_guid: Guid,
        pub(super) vendor_defined_data: [u8],
    }

    impl Vendor {
        /// Vendor-assigned GUID that defines the data that follows.
        pub fn vendor_guid(&self) -> Guid {
            self.vendor_guid
        }

        /// Vendor-defined data.
        pub fn vendor_defined_data(&self) -> &[u8] {
            &self.vendor_defined_data
        }
    }

    impl fmt::Debug for Vendor {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Vendor")
                .field("vendor_guid", &{ self.vendor_guid })
                .field("vendor_defined_data", {
                    let ptr = addr_of!(self.vendor_defined_data);
                    let (ptr, len) = ptr.to_raw_parts();
                    let byte_len = size_of::<u8>() * len;
                    unsafe { &slice::from_raw_parts(ptr.cast::<u8>(), byte_len) }
                })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Vendor {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            let static_size = 20usize;
            let dst_size = size_of_val(node)
                .checked_sub(static_size)
                .ok_or(NodeConversionError::InvalidLength)?;
            let elem_size = size_of::<u8>();
            if dst_size % elem_size != 0 {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Vendor = ptr::from_raw_parts(node.cast(), dst_size / elem_size);
            Ok(unsafe { &*node })
        }
    }

    /// Serial Attached SCSI (SAS) extended messaging device path node.
    #[repr(C, packed)]
    pub struct SasEx {
        pub(super) header: DevicePathHeader,
        pub(super) sas_address: [u8; 8usize],
        pub(super) logical_unit_number: [u8; 8usize],
        pub(super) info: u16,
        pub(super) relative_target_port: u16,
    }

    impl SasEx {
        /// SAS address.
        pub fn sas_address(&self) -> [u8; 8usize] {
            self.sas_address
        }

        /// Logical Unit Number.
        pub fn logical_unit_number(&self) -> [u8; 8usize] {
            self.logical_unit_number
        }

        /// Information about the device and its interconnect.
        pub fn info(&self) -> u16 {
            self.info
        }

        /// Relative Target Port (RTP).
        pub fn relative_target_port(&self) -> u16 {
            self.relative_target_port
        }
    }

    impl fmt::Debug for SasEx {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("SasEx")
                .field("sas_address", &{ self.sas_address })
                .field("logical_unit_number", &{ self.logical_unit_number })
                .field("info", &{ self.info })
                .field("relative_target_port", &{ self.relative_target_port })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &SasEx {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<SasEx>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const SasEx = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// iSCSI messaging device path node.
    #[repr(C, packed)]
    pub struct Iscsi {
        pub(super) header: DevicePathHeader,
        pub(super) protocol: crate::proto::device_path::messaging::IscsiProtocol,
        pub(super) options: crate::proto::device_path::messaging::IscsiLoginOptions,
        pub(super) logical_unit_number: [u8; 8usize],
        pub(super) target_portal_group_tag: u16,
        pub(super) iscsi_target_name: [u8],
    }

    impl Iscsi {
        /// Network protocol.
        pub fn protocol(&self) -> crate::proto::device_path::messaging::IscsiProtocol {
            self.protocol
        }

        /// iSCSI login options (bitfield).
        pub fn options(&self) -> crate::proto::device_path::messaging::IscsiLoginOptions {
            self.options
        }

        /// iSCSI Logical Unit Number.
        pub fn logical_unit_number(&self) -> [u8; 8usize] {
            self.logical_unit_number
        }

        /// iSCSI Target Portal group tag the initiator intends to
        /// establish a session with.
        pub fn target_portal_group_tag(&self) -> u16 {
            self.target_portal_group_tag
        }

        /// iSCSI Node Target name.
        ///
        /// The UEFI Specification does not specify how the string is
        /// encoded, but gives one example that appears to be
        /// null-terminated ASCII.
        pub fn iscsi_target_name(&self) -> &[u8] {
            &self.iscsi_target_name
        }
    }

    impl fmt::Debug for Iscsi {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Iscsi")
                .field("protocol", &{ self.protocol })
                .field("options", &{ self.options })
                .field("logical_unit_number", &{ self.logical_unit_number })
                .field("target_portal_group_tag", &{ self.target_portal_group_tag })
                .field("iscsi_target_name", {
                    let ptr = addr_of!(self.iscsi_target_name);
                    let (ptr, len) = ptr.to_raw_parts();
                    let byte_len = size_of::<u8>() * len;
                    unsafe { &slice::from_raw_parts(ptr.cast::<u8>(), byte_len) }
                })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Iscsi {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            let static_size = 18usize;
            let dst_size = size_of_val(node)
                .checked_sub(static_size)
                .ok_or(NodeConversionError::InvalidLength)?;
            let elem_size = size_of::<u8>();
            if dst_size % elem_size != 0 {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Iscsi = ptr::from_raw_parts(node.cast(), dst_size / elem_size);
            Ok(unsafe { &*node })
        }
    }

    /// NVM Express namespace messaging device path node.
    #[repr(C, packed)]
    pub struct NvmeNamespace {
        pub(super) header: DevicePathHeader,
        pub(super) namespace_identifier: u32,
        pub(super) ieee_extended_unique_identifier: u64,
    }

    impl NvmeNamespace {
        /// Namespace identifier (NSID). The values 0 and 0xffff_ffff
        /// are invalid.
        pub fn namespace_identifier(&self) -> u32 {
            self.namespace_identifier
        }

        /// IEEE Extended Unique Identifier (EUI-64), or 0 if the device
        /// does not have a EUI-64.
        pub fn ieee_extended_unique_identifier(&self) -> u64 {
            self.ieee_extended_unique_identifier
        }
    }

    impl fmt::Debug for NvmeNamespace {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("NvmeNamespace")
                .field("namespace_identifier", &{ self.namespace_identifier })
                .field("ieee_extended_unique_identifier", &{
                    self.ieee_extended_unique_identifier
                })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &NvmeNamespace {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<NvmeNamespace>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const NvmeNamespace = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// Uniform Resource Identifier (URI) messaging device path node.
    #[repr(C, packed)]
    pub struct Uri {
        pub(super) header: DevicePathHeader,
        pub(super) value: [u8],
    }

    impl Uri {
        /// URI as defined by [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986).
        pub fn value(&self) -> &[u8] {
            &self.value
        }
    }

    impl fmt::Debug for Uri {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Uri")
                .field("value", {
                    let ptr = addr_of!(self.value);
                    let (ptr, len) = ptr.to_raw_parts();
                    let byte_len = size_of::<u8>() * len;
                    unsafe { &slice::from_raw_parts(ptr.cast::<u8>(), byte_len) }
                })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Uri {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            let static_size = 4usize;
            let dst_size = size_of_val(node)
                .checked_sub(static_size)
                .ok_or(NodeConversionError::InvalidLength)?;
            let elem_size = size_of::<u8>();
            if dst_size % elem_size != 0 {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Uri = ptr::from_raw_parts(node.cast(), dst_size / elem_size);
            Ok(unsafe { &*node })
        }
    }

    /// Universal Flash Storage (UFS) messaging device path node.
    #[repr(C, packed)]
    pub struct Ufs {
        pub(super) header: DevicePathHeader,
        pub(super) target_id: u8,
        pub(super) logical_unit_number: u8,
    }

    impl Ufs {
        /// Target ID on the UFS interface (PUN).
        pub fn target_id(&self) -> u8 {
            self.target_id
        }

        /// Logical Unit Number (LUN).
        pub fn logical_unit_number(&self) -> u8 {
            self.logical_unit_number
        }
    }

    impl fmt::Debug for Ufs {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Ufs")
                .field("target_id", &{ self.target_id })
                .field("logical_unit_number", &{ self.logical_unit_number })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Ufs {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Ufs>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Ufs = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// Secure Digital (SD) messaging device path node.
    #[repr(C, packed)]
    pub struct Sd {
        pub(super) header: DevicePathHeader,
        pub(super) slot_number: u8,
    }

    impl Sd {
        /// Slot number.
        pub fn slot_number(&self) -> u8 {
            self.slot_number
        }
    }

    impl fmt::Debug for Sd {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Sd")
                .field("slot_number", &{ self.slot_number })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Sd {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Sd>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Sd = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// Bluetooth messaging device path node.
    #[repr(C, packed)]
    pub struct Bluetooth {
        pub(super) header: DevicePathHeader,
        pub(super) device_address: [u8; 6usize],
    }

    impl Bluetooth {
        /// 48-bit bluetooth device address.
        pub fn device_address(&self) -> [u8; 6usize] {
            self.device_address
        }
    }

    impl fmt::Debug for Bluetooth {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Bluetooth")
                .field("device_address", &{ self.device_address })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Bluetooth {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Bluetooth>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Bluetooth = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// Wi-Fi messaging device path node.
    #[repr(C, packed)]
    pub struct Wifi {
        pub(super) header: DevicePathHeader,
        pub(super) ssid: [u8; 32usize],
    }

    impl Wifi {
        /// Service set identifier (SSID).
        pub fn ssid(&self) -> [u8; 32usize] {
            self.ssid
        }
    }

    impl fmt::Debug for Wifi {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Wifi")
                .field("ssid", &{ self.ssid })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Wifi {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Wifi>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Wifi = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// Embedded Multi-Media Card (eMMC) messaging device path node.
    #[repr(C, packed)]
    pub struct Emmc {
        pub(super) header: DevicePathHeader,
        pub(super) slot_number: u8,
    }

    impl Emmc {
        /// Slot number.
        pub fn slot_number(&self) -> u8 {
            self.slot_number
        }
    }

    impl fmt::Debug for Emmc {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Emmc")
                .field("slot_number", &{ self.slot_number })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Emmc {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Emmc>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Emmc = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// BluetoothLE messaging device path node.
    #[repr(C, packed)]
    pub struct BluetoothLe {
        pub(super) header: DevicePathHeader,
        pub(super) device_address: [u8; 6usize],
        pub(super) address_type: crate::proto::device_path::messaging::BluetoothLeAddressType,
    }

    impl BluetoothLe {
        /// 48-bit bluetooth device address.
        pub fn device_address(&self) -> [u8; 6usize] {
            self.device_address
        }

        /// Address type.
        pub fn address_type(&self) -> crate::proto::device_path::messaging::BluetoothLeAddressType {
            self.address_type
        }
    }

    impl fmt::Debug for BluetoothLe {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("BluetoothLe")
                .field("device_address", &{ self.device_address })
                .field("address_type", &{ self.address_type })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &BluetoothLe {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<BluetoothLe>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const BluetoothLe = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// DNS messaging device path node.
    #[repr(C, packed)]
    pub struct Dns {
        pub(super) header: DevicePathHeader,
        pub(super) address_type: crate::proto::device_path::messaging::DnsAddressType,
        pub(super) addresses: [IpAddress],
    }

    impl Dns {
        /// Whether the addresses are IPv4 or IPv6.
        pub fn address_type(&self) -> crate::proto::device_path::messaging::DnsAddressType {
            self.address_type
        }

        /// One or more instances of the DNS server address.
        pub fn addresses(&self) -> UnalignedSlice<IpAddress> {
            let ptr: *const [IpAddress] = addr_of!(self.addresses);
            let (ptr, len): (*const (), usize) = ptr.to_raw_parts();
            unsafe { UnalignedSlice::new(ptr.cast::<IpAddress>(), len) }
        }
    }

    impl fmt::Debug for Dns {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Dns")
                .field("address_type", &{ self.address_type })
                .field("addresses", {
                    let ptr = addr_of!(self.addresses);
                    let (ptr, len) = ptr.to_raw_parts();
                    let byte_len = size_of::<IpAddress>() * len;
                    unsafe { &slice::from_raw_parts(ptr.cast::<u8>(), byte_len) }
                })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Dns {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            let static_size = 5usize;
            let dst_size = size_of_val(node)
                .checked_sub(static_size)
                .ok_or(NodeConversionError::InvalidLength)?;
            let elem_size = size_of::<IpAddress>();
            if dst_size % elem_size != 0 {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Dns = ptr::from_raw_parts(node.cast(), dst_size / elem_size);
            Ok(unsafe { &*node })
        }
    }

    /// NVDIMM namespace messaging device path node.
    #[repr(C, packed)]
    pub struct NvdimmNamespace {
        pub(super) header: DevicePathHeader,
        pub(super) uuid: [u8; 16usize],
    }

    impl NvdimmNamespace {
        /// Namespace unique label identifier.
        pub fn uuid(&self) -> [u8; 16usize] {
            self.uuid
        }
    }

    impl fmt::Debug for NvdimmNamespace {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("NvdimmNamespace")
                .field("uuid", &{ self.uuid })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &NvdimmNamespace {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<NvdimmNamespace>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const NvdimmNamespace = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// REST service messaging device path node.
    #[repr(C, packed)]
    pub struct RestService {
        pub(super) header: DevicePathHeader,
        pub(super) service_type: crate::proto::device_path::messaging::RestServiceType,
        pub(super) access_mode: crate::proto::device_path::messaging::RestServiceAccessMode,
        pub(super) vendor_guid_and_data: [u8],
    }

    impl RestService {
        /// Type of REST service.
        pub fn service_type(&self) -> crate::proto::device_path::messaging::RestServiceType {
            self.service_type
        }

        /// Whether the service is in-band or out-of-band.
        pub fn access_mode(&self) -> crate::proto::device_path::messaging::RestServiceAccessMode {
            self.access_mode
        }
    }

    impl fmt::Debug for RestService {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("RestService")
                .field("service_type", &{ self.service_type })
                .field("access_mode", &{ self.access_mode })
                .field("vendor_guid_and_data", {
                    let ptr = addr_of!(self.vendor_guid_and_data);
                    let (ptr, len) = ptr.to_raw_parts();
                    let byte_len = size_of::<u8>() * len;
                    unsafe { &slice::from_raw_parts(ptr.cast::<u8>(), byte_len) }
                })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &RestService {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            let static_size = 6usize;
            let dst_size = size_of_val(node)
                .checked_sub(static_size)
                .ok_or(NodeConversionError::InvalidLength)?;
            let elem_size = size_of::<u8>();
            if dst_size % elem_size != 0 {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const RestService = ptr::from_raw_parts(node.cast(), dst_size / elem_size);
            Ok(unsafe { &*node })
        }
    }

    /// NVME over Fabric (NVMe-oF) namespace messaging device path node.
    #[repr(C, packed)]
    pub struct NvmeOfNamespace {
        pub(super) header: DevicePathHeader,
        pub(super) nidt: u8,
        pub(super) nid: [u8; 16usize],
        pub(super) subsystem_nqn: [u8],
    }

    impl NvmeOfNamespace {
        /// Namespace Identifier Type (NIDT).
        pub fn nidt(&self) -> u8 {
            self.nidt
        }

        /// Namespace Identifier (NID).
        pub fn nid(&self) -> [u8; 16usize] {
            self.nid
        }

        /// Unique identifier of an NVM subsystem stored as a
        /// null-terminated UTF-8 string. Maximum length of 224 bytes.
        pub fn subsystem_nqn(&self) -> &[u8] {
            &self.subsystem_nqn
        }
    }

    impl fmt::Debug for NvmeOfNamespace {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("NvmeOfNamespace")
                .field("nidt", &{ self.nidt })
                .field("nid", &{ self.nid })
                .field("subsystem_nqn", {
                    let ptr = addr_of!(self.subsystem_nqn);
                    let (ptr, len) = ptr.to_raw_parts();
                    let byte_len = size_of::<u8>() * len;
                    unsafe { &slice::from_raw_parts(ptr.cast::<u8>(), byte_len) }
                })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &NvmeOfNamespace {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            let static_size = 21usize;
            let dst_size = size_of_val(node)
                .checked_sub(static_size)
                .ok_or(NodeConversionError::InvalidLength)?;
            let elem_size = size_of::<u8>();
            if dst_size % elem_size != 0 {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const NvmeOfNamespace =
                ptr::from_raw_parts(node.cast(), dst_size / elem_size);
            Ok(unsafe { &*node })
        }
    }

    newtype_enum! { # [doc = " Whether the ATAPI device is primary or secondary."] pub enum PrimarySecondary : u8 => { # [doc = " Primary."] PRIMARY = 0x00 , # [doc = " Secondary."] SECONDARY = 0x01 , }

    }

    newtype_enum! { # [doc = " Whether the ATAPI device is master or slave."] pub enum MasterSlave : u8 => { # [doc = " Master mode."] MASTER = 0x00 , # [doc = " Slave mode."] SLAVE = 0x01 , }

    }

    newtype_enum! { # [doc = " Origin of the source IP address."] pub enum Ipv4AddressOrigin : u8 => { # [doc = " Source IP address was assigned through DHCP."] DHCP = 0x00 , # [doc = " Source IP address is statically bound."] STATIC = 0x01 , }

    }

    newtype_enum! { # [doc = " Origin of the local IP address."] pub enum Ipv6AddressOrigin : u8 => { # [doc = " Local IP address was manually configured."] MANUAL = 0x00 , # [doc = " Local IP address assigned through IPv6 stateless"] # [doc = " auto-configuration."] STATELESS_AUTO_CONFIGURATION = 0x01 , # [doc = " Local IP address assigned through IPv6 stateful"] # [doc = " configuration."] STATEFUL_CONFIGURATION = 0x02 , }

    }

    bitflags! { # [doc = " Flags to identify/manage InfiniBand elements."] # [repr (transparent)] pub struct InfinibandResourceFlags : u32 { # [doc = " Set = service, unset = IOC."] const SERVICE = 0x0000_0001 ; # [doc = " Extended boot environment."] const EXTENDED_BOOT_ENVIRONMENT = 0x0000_0002 ; # [doc = " Console protocol."] const CONSOLE_PROTOCOL = 0x0000_0004 ; # [doc = " Storage protocol."] const STORAGE_PROTOCOL = 0x0000_0008 ; # [doc = " Network protocol."] const NETWORK_PROTOCOL = 0x0000_0010 ; }

    }

    newtype_enum! { # [doc = " UART parity setting."] pub enum Parity : u8 => { # [doc = " Default parity."] DEFAULT = 0x00 , # [doc = " No parity."] NO = 0x01 , # [doc = " Even parity."] EVEN = 0x02 , # [doc = " Odd parity."] ODD = 0x03 , # [doc = " Mark parity."] MARK = 0x04 , # [doc = " Space parity."] SPACE = 0x05 , }

    }

    newtype_enum! { # [doc = " UART number of stop bits."] pub enum StopBits : u8 => { # [doc = " Default number of stop bits."] DEFAULT = 0x00 , # [doc = " 1 stop bit."] ONE = 0x01 , # [doc = " 1.5 stop bits."] ONE_POINT_FIVE = 0x02 , # [doc = " 2 stop bits."] TWO = 0x03 , }

    }

    newtype_enum! { # [doc = " iSCSI network protocol."] pub enum IscsiProtocol : u16 => { # [doc = " TCP."] TCP = 0x0000 , }

    }

    bitflags! { # [doc = " iSCSI login options."] # [repr (transparent)] pub struct IscsiLoginOptions : u16 { # [doc = " Header digest using CRC32. If not set, no header digest."] const HEADER_DIGEST_USING_CRC32 = 0x0002 ; # [doc = " Data digest using CRC32. If not set, no data digest."] const DATA_DIGEST_USING_CRC32 = 0x0008 ; # [doc = " Auth method none. If not set, auth method CHAP."] const AUTH_METHOD_NONE = 0x0800 ; # [doc = " CHAP UNI. If not set, CHAP BI."] const CHAP_UNI = 0x1000 ; }

    }

    newtype_enum! { # [doc = " BluetoothLE address type."] pub enum BluetoothLeAddressType : u8 => { # [doc = " Public device address."] PUBLIC = 0x00 , # [doc = " Random device address."] RANDOM = 0x01 , }

    }

    newtype_enum! { # [doc = " Whether the address is IPv4 or IPv6."] pub enum DnsAddressType : u8 => { # [doc = " DNS server address is IPv4."] IPV4 = 0x00 , # [doc = " DNS server address is IPv6."] IPV6 = 0x01 , }

    }

    impl RestService {
        /// Get the vendor GUID and vendor data. Only used if the
        /// service type is [`VENDOR`], otherwise returns None.
        ///
        /// [`VENDOR`]: uefi::proto::device_path::messaging::RestServiceType
        pub fn vendor_guid_and_data(&self) -> Option<(Guid, &[u8])> {
            if self.service_type == RestServiceType::VENDOR
                && self.vendor_guid_and_data.len() >= size_of::<Guid>()
            {
                let (guid, data) = self.vendor_guid_and_data.split_at(size_of::<Guid>());
                let guid: [u8; 16] = guid.try_into().unwrap();
                Some((Guid::from_bytes(guid), data))
            } else {
                None
            }
        }
    }

    newtype_enum! { # [doc = " Type of REST service."] pub enum RestServiceType : u8 => { # [doc = " Redfish REST service."] REDFISH = 0x01 , # [doc = " OData REST service."] ODATA = 0x02 , # [doc = " Vendor-specific REST service."] VENDOR = 0xff , }

    }

    newtype_enum! { # [doc = " Whether the service is in-band or out-of-band."] pub enum RestServiceAccessMode : u8 => { # [doc = " In-band REST service."] IN_BAND = 0x01 , # [doc = " Out-of-band REST service."] OUT_OF_BAND = 0x02 , }

    }
}

/// Device path nodes for [`DeviceType::MEDIA`].
pub mod media {
    use super::*;
    /// Hard drive media device path node.
    #[repr(C, packed)]
    pub struct HardDrive {
        pub(super) header: DevicePathHeader,
        pub(super) partition_number: u32,
        pub(super) partition_start: u64,
        pub(super) partition_size: u64,
        pub(super) partition_signature: [u8; 16usize],
        pub(super) partition_format: crate::proto::device_path::media::PartitionFormat,
        pub(super) signature_type: u8,
    }

    impl HardDrive {
        /// Index of the partition, starting from 1.
        pub fn partition_number(&self) -> u32 {
            self.partition_number
        }

        /// Starting LBA (logical block address) of the partition.
        pub fn partition_start(&self) -> u64 {
            self.partition_start
        }

        /// Size of the partition in blocks.
        pub fn partition_size(&self) -> u64 {
            self.partition_size
        }

        /// Partition format.
        pub fn partition_format(&self) -> crate::proto::device_path::media::PartitionFormat {
            self.partition_format
        }
    }

    impl fmt::Debug for HardDrive {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("HardDrive")
                .field("partition_number", &{ self.partition_number })
                .field("partition_start", &{ self.partition_start })
                .field("partition_size", &{ self.partition_size })
                .field("partition_signature", &{ self.partition_signature })
                .field("partition_format", &{ self.partition_format })
                .field("signature_type", &{ self.signature_type })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &HardDrive {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<HardDrive>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const HardDrive = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// CD-ROM media device path node.
    #[repr(C, packed)]
    pub struct CdRom {
        pub(super) header: DevicePathHeader,
        pub(super) boot_entry: u32,
        pub(super) partition_start: u64,
        pub(super) partition_size: u64,
    }

    impl CdRom {
        /// Boot entry number from the boot catalog, or 0 for the
        /// default entry.
        pub fn boot_entry(&self) -> u32 {
            self.boot_entry
        }

        /// Starting RBA (Relative logical Block Address).
        pub fn partition_start(&self) -> u64 {
            self.partition_start
        }

        /// Size of the partition in blocks.
        pub fn partition_size(&self) -> u64 {
            self.partition_size
        }
    }

    impl fmt::Debug for CdRom {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("CdRom")
                .field("boot_entry", &{ self.boot_entry })
                .field("partition_start", &{ self.partition_start })
                .field("partition_size", &{ self.partition_size })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &CdRom {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<CdRom>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const CdRom = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// Vendor-defined media device path node.
    #[repr(C, packed)]
    pub struct Vendor {
        pub(super) header: DevicePathHeader,
        pub(super) vendor_guid: Guid,
        pub(super) vendor_defined_data: [u8],
    }

    impl Vendor {
        /// Vendor-assigned GUID that defines the data that follows.
        pub fn vendor_guid(&self) -> Guid {
            self.vendor_guid
        }

        /// Vendor-defined data.
        pub fn vendor_defined_data(&self) -> &[u8] {
            &self.vendor_defined_data
        }
    }

    impl fmt::Debug for Vendor {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Vendor")
                .field("vendor_guid", &{ self.vendor_guid })
                .field("vendor_defined_data", {
                    let ptr = addr_of!(self.vendor_defined_data);
                    let (ptr, len) = ptr.to_raw_parts();
                    let byte_len = size_of::<u8>() * len;
                    unsafe { &slice::from_raw_parts(ptr.cast::<u8>(), byte_len) }
                })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Vendor {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            let static_size = 20usize;
            let dst_size = size_of_val(node)
                .checked_sub(static_size)
                .ok_or(NodeConversionError::InvalidLength)?;
            let elem_size = size_of::<u8>();
            if dst_size % elem_size != 0 {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Vendor = ptr::from_raw_parts(node.cast(), dst_size / elem_size);
            Ok(unsafe { &*node })
        }
    }

    /// File path media device path node.
    #[repr(C, packed)]
    pub struct FilePath {
        pub(super) header: DevicePathHeader,
        pub(super) path_name: [u16],
    }

    impl FilePath {
        /// Null-terminated path.
        pub fn path_name(&self) -> UnalignedSlice<u16> {
            let ptr: *const [u16] = addr_of!(self.path_name);
            let (ptr, len): (*const (), usize) = ptr.to_raw_parts();
            unsafe { UnalignedSlice::new(ptr.cast::<u16>(), len) }
        }
    }

    impl fmt::Debug for FilePath {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("FilePath")
                .field("path_name", {
                    let ptr = addr_of!(self.path_name);
                    let (ptr, len) = ptr.to_raw_parts();
                    let byte_len = size_of::<u16>() * len;
                    unsafe { &slice::from_raw_parts(ptr.cast::<u8>(), byte_len) }
                })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &FilePath {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            let static_size = 4usize;
            let dst_size = size_of_val(node)
                .checked_sub(static_size)
                .ok_or(NodeConversionError::InvalidLength)?;
            let elem_size = size_of::<u16>();
            if dst_size % elem_size != 0 {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const FilePath = ptr::from_raw_parts(node.cast(), dst_size / elem_size);
            Ok(unsafe { &*node })
        }
    }

    /// Media protocol media device path node.
    #[repr(C, packed)]
    pub struct Protocol {
        pub(super) header: DevicePathHeader,
        pub(super) protocol_guid: Guid,
    }

    impl Protocol {
        /// The ID of the protocol.
        pub fn protocol_guid(&self) -> Guid {
            self.protocol_guid
        }
    }

    impl fmt::Debug for Protocol {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Protocol")
                .field("protocol_guid", &{ self.protocol_guid })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &Protocol {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<Protocol>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const Protocol = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// PIWG firmware file media device path node.
    #[repr(C, packed)]
    pub struct PiwgFirmwareFile {
        pub(super) header: DevicePathHeader,
        pub(super) data: [u8],
    }

    impl PiwgFirmwareFile {
        /// Contents are defined in the UEFI PI Specification.
        pub fn data(&self) -> &[u8] {
            &self.data
        }
    }

    impl fmt::Debug for PiwgFirmwareFile {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("PiwgFirmwareFile")
                .field("data", {
                    let ptr = addr_of!(self.data);
                    let (ptr, len) = ptr.to_raw_parts();
                    let byte_len = size_of::<u8>() * len;
                    unsafe { &slice::from_raw_parts(ptr.cast::<u8>(), byte_len) }
                })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &PiwgFirmwareFile {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            let static_size = 4usize;
            let dst_size = size_of_val(node)
                .checked_sub(static_size)
                .ok_or(NodeConversionError::InvalidLength)?;
            let elem_size = size_of::<u8>();
            if dst_size % elem_size != 0 {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const PiwgFirmwareFile =
                ptr::from_raw_parts(node.cast(), dst_size / elem_size);
            Ok(unsafe { &*node })
        }
    }

    /// PIWG firmware volume media device path node.
    #[repr(C, packed)]
    pub struct PiwgFirmwareVolume {
        pub(super) header: DevicePathHeader,
        pub(super) data: [u8],
    }

    impl PiwgFirmwareVolume {
        /// Contents are defined in the UEFI PI Specification.
        pub fn data(&self) -> &[u8] {
            &self.data
        }
    }

    impl fmt::Debug for PiwgFirmwareVolume {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("PiwgFirmwareVolume")
                .field("data", {
                    let ptr = addr_of!(self.data);
                    let (ptr, len) = ptr.to_raw_parts();
                    let byte_len = size_of::<u8>() * len;
                    unsafe { &slice::from_raw_parts(ptr.cast::<u8>(), byte_len) }
                })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &PiwgFirmwareVolume {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            let static_size = 4usize;
            let dst_size = size_of_val(node)
                .checked_sub(static_size)
                .ok_or(NodeConversionError::InvalidLength)?;
            let elem_size = size_of::<u8>();
            if dst_size % elem_size != 0 {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const PiwgFirmwareVolume =
                ptr::from_raw_parts(node.cast(), dst_size / elem_size);
            Ok(unsafe { &*node })
        }
    }

    /// Relative offset range media device path node.
    #[repr(C, packed)]
    pub struct RelativeOffsetRange {
        pub(super) header: DevicePathHeader,
        pub(super) _reserved: u32,
        pub(super) starting_offset: u64,
        pub(super) ending_offset: u64,
    }

    impl RelativeOffsetRange {
        /// Offset of the first byte, relative to the parent device node.
        pub fn starting_offset(&self) -> u64 {
            self.starting_offset
        }

        /// Offset of the last byte, relative to the parent device node.
        pub fn ending_offset(&self) -> u64 {
            self.ending_offset
        }
    }

    impl fmt::Debug for RelativeOffsetRange {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("RelativeOffsetRange")
                .field("_reserved", &{ self._reserved })
                .field("starting_offset", &{ self.starting_offset })
                .field("ending_offset", &{ self.ending_offset })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &RelativeOffsetRange {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<RelativeOffsetRange>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const RelativeOffsetRange = node.cast();
            Ok(unsafe { &*node })
        }
    }

    /// RAM disk media device path node.
    #[repr(C, packed)]
    pub struct RamDisk {
        pub(super) header: DevicePathHeader,
        pub(super) starting_address: u64,
        pub(super) ending_address: u64,
        pub(super) disk_type: crate::proto::device_path::media::RamDiskType,
        pub(super) disk_instance: u16,
    }

    impl RamDisk {
        /// Starting memory address.
        pub fn starting_address(&self) -> u64 {
            self.starting_address
        }

        /// Ending memory address.
        pub fn ending_address(&self) -> u64 {
            self.ending_address
        }

        /// Type of RAM disk.
        pub fn disk_type(&self) -> crate::proto::device_path::media::RamDiskType {
            self.disk_type
        }

        /// RAM disk instance number if supported, otherwise 0.
        pub fn disk_instance(&self) -> u16 {
            self.disk_instance
        }
    }

    impl fmt::Debug for RamDisk {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("RamDisk")
                .field("starting_address", &{ self.starting_address })
                .field("ending_address", &{ self.ending_address })
                .field("disk_type", &{ self.disk_type })
                .field("disk_instance", &{ self.disk_instance })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &RamDisk {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            if size_of_val(node) != size_of::<RamDisk>() {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const RamDisk = node.cast();
            Ok(unsafe { &*node })
        }
    }

    impl HardDrive {
        /// Signature unique to this partition.
        pub fn partition_signature(&self) -> PartitionSignature {
            match self.signature_type {
                0 => PartitionSignature::None,
                1 => PartitionSignature::Mbr([
                    self.partition_signature[0],
                    self.partition_signature[1],
                    self.partition_signature[2],
                    self.partition_signature[3],
                ]),
                2 => PartitionSignature::Guid(Guid::from_bytes(self.partition_signature)),
                unknown => PartitionSignature::Unknown {
                    signature_type: unknown,
                    signature: self.partition_signature,
                },
            }
        }
    }

    /// Hard drive partition signature.
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
    pub enum PartitionSignature {
        /// No disk signature.
        None,
        /// 32-bit MBR partition signature.
        Mbr([u8; 4]),
        /// 128-bit GUID partition signature.
        Guid(Guid),
        /// Unknown signature type not listed in the UEFI Specification.
        Unknown {
            /// Signature type.
            signature_type: u8,
            /// Signature data.
            signature: [u8; 16],
        },
    }

    newtype_enum! { # [doc = " Hard drive partition format."] pub enum PartitionFormat : u8 => { # [doc = " MBR (PC-AT compatible Master Boot Record) format."] MBR = 0x01 , # [doc = " GPT (GUID Partition Table) format."] GPT = 0x02 , }

    }

    newtype_enum! { # [doc = " RAM disk type."] pub enum RamDiskType : Guid => { # [doc = " RAM disk with a raw disk format in volatile memory."] VIRTUAL_DISK = guid ! ("77ab535a-45fc-624b-5560-f7b281d1f96e") , # [doc = " RAM disk of an ISO image in volatile memory."] VIRTUAL_CD = guid ! ("3d5abd30-4175-87ce-6d64-d2ade523c4bb") , # [doc = " RAM disk with a raw disk format in persistent memory."] PERSISTENT_VIRTUAL_DISK = guid ! ("5cea02c9-4d07-69d3-269f-4496fbe096f9") , # [doc = " RAM disk of an ISO image in persistent memory."] PERSISTENT_VIRTUAL_CD = guid ! ("08018188-42cd-bb48-100f-5387d53ded3d") , }

    }
}

/// Device path nodes for [`DeviceType::BIOS_BOOT_SPEC`].
pub mod bios_boot_spec {
    use super::*;
    /// BIOS Boot Specification device path node.
    #[repr(C, packed)]
    pub struct BootSpecification {
        pub(super) header: DevicePathHeader,
        pub(super) device_type: u16,
        pub(super) status_flag: u16,
        pub(super) description_string: [u8],
    }

    impl BootSpecification {
        /// Device type as defined by the BIOS Boot Specification.
        pub fn device_type(&self) -> u16 {
            self.device_type
        }

        /// Status flags as defined by the BIOS Boot Specification.
        pub fn status_flag(&self) -> u16 {
            self.status_flag
        }

        /// Description of the boot device encoded as a null-terminated
        /// ASCII string.
        pub fn description_string(&self) -> &[u8] {
            &self.description_string
        }
    }

    impl fmt::Debug for BootSpecification {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("BootSpecification")
                .field("device_type", &{ self.device_type })
                .field("status_flag", &{ self.status_flag })
                .field("description_string", {
                    let ptr = addr_of!(self.description_string);
                    let (ptr, len) = ptr.to_raw_parts();
                    let byte_len = size_of::<u8>() * len;
                    unsafe { &slice::from_raw_parts(ptr.cast::<u8>(), byte_len) }
                })
                .finish()
        }
    }

    impl TryFrom<&DevicePathNode> for &BootSpecification {
        type Error = NodeConversionError;
        fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
            let static_size = 8usize;
            let dst_size = size_of_val(node)
                .checked_sub(static_size)
                .ok_or(NodeConversionError::InvalidLength)?;
            let elem_size = size_of::<u8>();
            if dst_size % elem_size != 0 {
                return Err(NodeConversionError::InvalidLength);
            }

            let node: *const DevicePathNode = node;
            let node: *const BootSpecification =
                ptr::from_raw_parts(node.cast(), dst_size / elem_size);
            Ok(unsafe { &*node })
        }
    }
}

/// Enum of references to all the different device path node
/// types. Return type of [`DevicePathNode::as_enum`].
pub enum DevicePathNodeEnum<'a> {
    /// Node that terminates a [`DevicePathInstance`].
    ///
    /// [`DevicePathInstance`]: crate::proto::device_path::DevicePathInstance
    EndInstance(&'a end::Instance),
    /// Node that terminates an entire [`DevicePath`].
    ///
    /// [`DevicePath`]: crate::proto::device_path::DevicePath
    EndEntire(&'a end::Entire),
    /// PCI hardware device path node.
    HardwarePci(&'a hardware::Pci),
    /// PCCARD hardware device path node.
    HardwarePccard(&'a hardware::Pccard),
    /// Memory mapped hardware device path node.
    HardwareMemoryMapped(&'a hardware::MemoryMapped),
    /// Vendor-defined hardware device path node.
    HardwareVendor(&'a hardware::Vendor),
    /// Controller hardware device path node.
    HardwareController(&'a hardware::Controller),
    /// Baseboard Management Controller (BMC) host interface hardware
    /// device path node.
    HardwareBmc(&'a hardware::Bmc),
    /// ACPI device path node.
    AcpiAcpi(&'a acpi::Acpi),
    /// Expanded ACPI device path node.
    AcpiExpanded(&'a acpi::Expanded),
    /// ADR ACPI device path node.
    AcpiAdr(&'a acpi::Adr),
    /// NVDIMM ACPI device path node.
    AcpiNvdimm(&'a acpi::Nvdimm),
    /// ATAPI messaging device path node.
    MessagingAtapi(&'a messaging::Atapi),
    /// SCSI messaging device path node.
    MessagingScsi(&'a messaging::Scsi),
    /// Fibre channel messaging device path node.
    MessagingFibreChannel(&'a messaging::FibreChannel),
    /// Fibre channel extended messaging device path node.
    MessagingFibreChannelEx(&'a messaging::FibreChannelEx),
    /// 1394 messaging device path node.
    MessagingIeee1394(&'a messaging::Ieee1394),
    /// USB messaging device path node.
    MessagingUsb(&'a messaging::Usb),
    /// SATA messaging device path node.
    MessagingSata(&'a messaging::Sata),
    /// USB World Wide ID (WWID) messaging device path node.
    MessagingUsbWwid(&'a messaging::UsbWwid),
    /// Device logical unit messaging device path node.
    MessagingDeviceLogicalUnit(&'a messaging::DeviceLogicalUnit),
    /// USB class messaging device path node.
    MessagingUsbClass(&'a messaging::UsbClass),
    /// I2O messaging device path node.
    MessagingI2o(&'a messaging::I2o),
    /// MAC address messaging device path node.
    MessagingMacAddress(&'a messaging::MacAddress),
    /// IPv4 messaging device path node.
    MessagingIpv4(&'a messaging::Ipv4),
    /// IPv6 messaging device path node.
    MessagingIpv6(&'a messaging::Ipv6),
    /// VLAN messaging device path node.
    MessagingVlan(&'a messaging::Vlan),
    /// InfiniBand messaging device path node.
    MessagingInfiniband(&'a messaging::Infiniband),
    /// UART messaging device path node.
    MessagingUart(&'a messaging::Uart),
    /// Vendor-defined messaging device path node.
    MessagingVendor(&'a messaging::Vendor),
    /// Serial Attached SCSI (SAS) extended messaging device path node.
    MessagingSasEx(&'a messaging::SasEx),
    /// iSCSI messaging device path node.
    MessagingIscsi(&'a messaging::Iscsi),
    /// NVM Express namespace messaging device path node.
    MessagingNvmeNamespace(&'a messaging::NvmeNamespace),
    /// Uniform Resource Identifier (URI) messaging device path node.
    MessagingUri(&'a messaging::Uri),
    /// Universal Flash Storage (UFS) messaging device path node.
    MessagingUfs(&'a messaging::Ufs),
    /// Secure Digital (SD) messaging device path node.
    MessagingSd(&'a messaging::Sd),
    /// Bluetooth messaging device path node.
    MessagingBluetooth(&'a messaging::Bluetooth),
    /// Wi-Fi messaging device path node.
    MessagingWifi(&'a messaging::Wifi),
    /// Embedded Multi-Media Card (eMMC) messaging device path node.
    MessagingEmmc(&'a messaging::Emmc),
    /// BluetoothLE messaging device path node.
    MessagingBluetoothLe(&'a messaging::BluetoothLe),
    /// DNS messaging device path node.
    MessagingDns(&'a messaging::Dns),
    /// NVDIMM namespace messaging device path node.
    MessagingNvdimmNamespace(&'a messaging::NvdimmNamespace),
    /// REST service messaging device path node.
    MessagingRestService(&'a messaging::RestService),
    /// NVME over Fabric (NVMe-oF) namespace messaging device path node.
    MessagingNvmeOfNamespace(&'a messaging::NvmeOfNamespace),
    /// Hard drive media device path node.
    MediaHardDrive(&'a media::HardDrive),
    /// CD-ROM media device path node.
    MediaCdRom(&'a media::CdRom),
    /// Vendor-defined media device path node.
    MediaVendor(&'a media::Vendor),
    /// File path media device path node.
    MediaFilePath(&'a media::FilePath),
    /// Media protocol media device path node.
    MediaProtocol(&'a media::Protocol),
    /// PIWG firmware file media device path node.
    MediaPiwgFirmwareFile(&'a media::PiwgFirmwareFile),
    /// PIWG firmware volume media device path node.
    MediaPiwgFirmwareVolume(&'a media::PiwgFirmwareVolume),
    /// Relative offset range media device path node.
    MediaRelativeOffsetRange(&'a media::RelativeOffsetRange),
    /// RAM disk media device path node.
    MediaRamDisk(&'a media::RamDisk),
    /// BIOS Boot Specification device path node.
    BiosBootSpecBootSpecification(&'a bios_boot_spec::BootSpecification),
}

impl<'a> TryFrom<&DevicePathNode> for DevicePathNodeEnum<'a> {
    type Error = NodeConversionError;
    fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
        Ok(match node.full_type() {
            (DeviceType::END, DeviceSubType::END_INSTANCE) => Self::EndInstance(node.try_into()?),
            (DeviceType::END, DeviceSubType::END_ENTIRE) => Self::EndEntire(node.try_into()?),
            (DeviceType::HARDWARE, DeviceSubType::HARDWARE_PCI) => {
                Self::HardwarePci(node.try_into()?)
            }
            (DeviceType::HARDWARE, DeviceSubType::HARDWARE_PCCARD) => {
                Self::HardwarePccard(node.try_into()?)
            }
            (DeviceType::HARDWARE, DeviceSubType::HARDWARE_MEMORY_MAPPED) => {
                Self::HardwareMemoryMapped(node.try_into()?)
            }
            (DeviceType::HARDWARE, DeviceSubType::HARDWARE_VENDOR) => {
                Self::HardwareVendor(node.try_into()?)
            }
            (DeviceType::HARDWARE, DeviceSubType::HARDWARE_CONTROLLER) => {
                Self::HardwareController(node.try_into()?)
            }
            (DeviceType::HARDWARE, DeviceSubType::HARDWARE_BMC) => {
                Self::HardwareBmc(node.try_into()?)
            }
            (DeviceType::ACPI, DeviceSubType::ACPI) => Self::AcpiAcpi(node.try_into()?),
            (DeviceType::ACPI, DeviceSubType::ACPI_EXPANDED) => {
                Self::AcpiExpanded(node.try_into()?)
            }
            (DeviceType::ACPI, DeviceSubType::ACPI_ADR) => Self::AcpiAdr(node.try_into()?),
            (DeviceType::ACPI, DeviceSubType::ACPI_NVDIMM) => Self::AcpiNvdimm(node.try_into()?),
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_ATAPI) => {
                Self::MessagingAtapi(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_SCSI) => {
                Self::MessagingScsi(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_FIBRE_CHANNEL) => {
                Self::MessagingFibreChannel(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_FIBRE_CHANNEL_EX) => {
                Self::MessagingFibreChannelEx(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_1394) => {
                Self::MessagingIeee1394(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_USB) => {
                Self::MessagingUsb(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_SATA) => {
                Self::MessagingSata(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_USB_WWID) => {
                Self::MessagingUsbWwid(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_DEVICE_LOGICAL_UNIT) => {
                Self::MessagingDeviceLogicalUnit(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_USB_CLASS) => {
                Self::MessagingUsbClass(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_I2O) => {
                Self::MessagingI2o(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_MAC_ADDRESS) => {
                Self::MessagingMacAddress(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_IPV4) => {
                Self::MessagingIpv4(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_IPV6) => {
                Self::MessagingIpv6(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_VLAN) => {
                Self::MessagingVlan(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_INFINIBAND) => {
                Self::MessagingInfiniband(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_UART) => {
                Self::MessagingUart(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_VENDOR) => {
                Self::MessagingVendor(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_SCSI_SAS_EX) => {
                Self::MessagingSasEx(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_ISCSI) => {
                Self::MessagingIscsi(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_NVME_NAMESPACE) => {
                Self::MessagingNvmeNamespace(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_URI) => {
                Self::MessagingUri(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_UFS) => {
                Self::MessagingUfs(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_SD) => {
                Self::MessagingSd(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_BLUETOOTH) => {
                Self::MessagingBluetooth(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_WIFI) => {
                Self::MessagingWifi(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_EMMC) => {
                Self::MessagingEmmc(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_BLUETOOTH_LE) => {
                Self::MessagingBluetoothLe(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_DNS) => {
                Self::MessagingDns(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_NVDIMM_NAMESPACE) => {
                Self::MessagingNvdimmNamespace(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_REST_SERVICE) => {
                Self::MessagingRestService(node.try_into()?)
            }
            (DeviceType::MESSAGING, DeviceSubType::MESSAGING_NVME_OF_NAMESPACE) => {
                Self::MessagingNvmeOfNamespace(node.try_into()?)
            }
            (DeviceType::MEDIA, DeviceSubType::MEDIA_HARD_DRIVE) => {
                Self::MediaHardDrive(node.try_into()?)
            }
            (DeviceType::MEDIA, DeviceSubType::MEDIA_CD_ROM) => Self::MediaCdRom(node.try_into()?),
            (DeviceType::MEDIA, DeviceSubType::MEDIA_VENDOR) => Self::MediaVendor(node.try_into()?),
            (DeviceType::MEDIA, DeviceSubType::MEDIA_FILE_PATH) => {
                Self::MediaFilePath(node.try_into()?)
            }
            (DeviceType::MEDIA, DeviceSubType::MEDIA_PROTOCOL) => {
                Self::MediaProtocol(node.try_into()?)
            }
            (DeviceType::MEDIA, DeviceSubType::MEDIA_PIWG_FIRMWARE_FILE) => {
                Self::MediaPiwgFirmwareFile(node.try_into()?)
            }
            (DeviceType::MEDIA, DeviceSubType::MEDIA_PIWG_FIRMWARE_VOLUME) => {
                Self::MediaPiwgFirmwareVolume(node.try_into()?)
            }
            (DeviceType::MEDIA, DeviceSubType::MEDIA_RELATIVE_OFFSET_RANGE) => {
                Self::MediaRelativeOffsetRange(node.try_into()?)
            }
            (DeviceType::MEDIA, DeviceSubType::MEDIA_RAM_DISK) => {
                Self::MediaRamDisk(node.try_into()?)
            }
            (DeviceType::BIOS_BOOT_SPEC, DeviceSubType::BIOS_BOOT_SPECIFICATION) => {
                Self::BiosBootSpecBootSpecification(node.try_into()?)
            }
            _ => return Err(NodeConversionError::UnsupportedType),
        })
    }
}

/// Build device paths from their component nodes.
pub mod build {
    use super::*;
    use crate::proto::device_path::build::{BuildError, BuildNode};
    use crate::proto::device_path::{DeviceSubType, DeviceType};
    use crate::CStr16;
    use core::mem::{size_of_val, MaybeUninit};
    /// Device path build nodes for [`DeviceType::END`].
    pub mod end {
        use super::*;
        /// Node that terminates a [`DevicePathInstance`].
        ///
        /// [`DevicePathInstance`]: crate::proto::device_path::DevicePathInstance
        pub struct Instance;
        unsafe impl BuildNode for Instance {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 4usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::END,
                            sub_type: DeviceSubType::END_INSTANCE,
                            length: u16::try_from(size).unwrap(),
                        });
                }
            }
        }

        /// Node that terminates an entire [`DevicePath`].
        ///
        /// [`DevicePath`]: crate::proto::device_path::DevicePath
        pub struct Entire;
        unsafe impl BuildNode for Entire {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 4usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::END,
                            sub_type: DeviceSubType::END_ENTIRE,
                            length: u16::try_from(size).unwrap(),
                        });
                }
            }
        }
    }

    /// Device path build nodes for [`DeviceType::HARDWARE`].
    pub mod hardware {
        use super::*;
        /// PCI hardware device path node.
        pub struct Pci {
            /// PCI function number.
            pub function: u8,
            /// PCI device number.
            pub device: u8,
        }

        unsafe impl BuildNode for Pci {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 6usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::HARDWARE,
                            sub_type: DeviceSubType::HARDWARE_PCI,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<u8>()
                        .write_unaligned(self.function);
                    out_ptr
                        .add(5usize)
                        .cast::<u8>()
                        .write_unaligned(self.device);
                }
            }
        }

        /// PCCARD hardware device path node.
        pub struct Pccard {
            /// Function number starting from 0.
            pub function: u8,
        }

        unsafe impl BuildNode for Pccard {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 5usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::HARDWARE,
                            sub_type: DeviceSubType::HARDWARE_PCCARD,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<u8>()
                        .write_unaligned(self.function);
                }
            }
        }

        /// Memory mapped hardware device path node.
        pub struct MemoryMapped {
            /// Memory type.
            pub memory_type: MemoryType,
            /// Starting memory address.
            pub start_address: u64,
            /// Ending memory address.
            pub end_address: u64,
        }

        unsafe impl BuildNode for MemoryMapped {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 24usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::HARDWARE,
                            sub_type: DeviceSubType::HARDWARE_MEMORY_MAPPED,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<MemoryType>()
                        .write_unaligned(self.memory_type);
                    out_ptr
                        .add(8usize)
                        .cast::<u64>()
                        .write_unaligned(self.start_address);
                    out_ptr
                        .add(16usize)
                        .cast::<u64>()
                        .write_unaligned(self.end_address);
                }
            }
        }

        /// Vendor-defined hardware device path node.
        pub struct Vendor<'a> {
            /// Vendor-assigned GUID that defines the data that follows.
            pub vendor_guid: Guid,
            /// Vendor-defined data.
            pub vendor_defined_data: &'a [u8],
        }

        unsafe impl<'a> BuildNode for Vendor<'a> {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 20usize + size_of_val(self.vendor_defined_data);
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::HARDWARE,
                            sub_type: DeviceSubType::HARDWARE_VENDOR,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<Guid>()
                        .write_unaligned(self.vendor_guid);
                    self.vendor_defined_data
                        .as_ptr()
                        .cast::<u8>()
                        .copy_to_nonoverlapping(
                            out_ptr.add(20usize),
                            size_of_val(self.vendor_defined_data),
                        );
                }
            }
        }

        /// Controller hardware device path node.
        pub struct Controller {
            /// Controller number.
            pub controller_number: u32,
        }

        unsafe impl BuildNode for Controller {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 8usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::HARDWARE,
                            sub_type: DeviceSubType::HARDWARE_CONTROLLER,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<u32>()
                        .write_unaligned(self.controller_number);
                }
            }
        }

        /// Baseboard Management Controller (BMC) host interface hardware
        /// device path node.
        pub struct Bmc {
            /// Host interface type.
            pub interface_type: crate::proto::device_path::hardware::BmcInterfaceType,
            /// Base address of the BMC. If the least-significant bit of the
            /// field is a 1 then the address is in I/O space, otherwise the
            /// address is memory-mapped.
            pub base_address: u64,
        }

        unsafe impl BuildNode for Bmc {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 13usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::HARDWARE,
                            sub_type: DeviceSubType::HARDWARE_BMC,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<crate::proto::device_path::hardware::BmcInterfaceType>()
                        .write_unaligned(self.interface_type);
                    out_ptr
                        .add(5usize)
                        .cast::<u64>()
                        .write_unaligned(self.base_address);
                }
            }
        }
    }

    /// Device path build nodes for [`DeviceType::ACPI`].
    pub mod acpi {
        use super::*;
        /// ACPI device path node.
        pub struct Acpi {
            /// Device's PnP hardware ID stored in a numeric 32-bit
            /// compressed EISA-type ID.
            pub hid: u32,
            /// Unique ID that is required by ACPI if two devices have the
            /// same HID.
            pub uid: u32,
        }

        unsafe impl BuildNode for Acpi {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 12usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::ACPI,
                            sub_type: DeviceSubType::ACPI,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr.add(4usize).cast::<u32>().write_unaligned(self.hid);
                    out_ptr.add(8usize).cast::<u32>().write_unaligned(self.uid);
                }
            }
        }

        /// Expanded ACPI device path node.
        pub struct Expanded<'a> {
            /// Device's PnP hardware ID stored in a numeric 32-bit compressed
            /// EISA-type ID.
            pub hid: u32,
            /// Unique ID that is required by ACPI if two devices have the
            /// same HID.
            pub uid: u32,
            /// Device's compatible PnP hardware ID stored in a numeric 32-bit
            /// compressed EISA-type ID.
            pub cid: u32,
            /// Device's PnP hardware ID stored as a null-terminated ASCII
            /// string. This value must match the corresponding HID in the
            /// ACPI name space. If the length of this string not including
            /// the null-terminator is 0, then the numeric HID is used.
            pub hid_str: &'a [u8],
            /// Unique ID that is required by ACPI if two devices have the
            /// same HID. This value is stored as a null-terminated ASCII
            /// string. If the length of this string not including the
            /// null-terminator is 0, then the numeric UID is used.
            pub uid_str: &'a [u8],
            /// Device's compatible PnP hardware ID stored as a
            /// null-terminated ASCII string. If the length of this string
            /// not including the null-terminator is 0, then the numeric CID
            /// is used.
            pub cid_str: &'a [u8],
        }

        unsafe impl<'a> BuildNode for Expanded<'a> {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 16usize
                    + size_of_val(self.hid_str)
                    + size_of_val(self.uid_str)
                    + size_of_val(self.cid_str);
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::ACPI,
                            sub_type: DeviceSubType::ACPI_EXPANDED,
                            length: u16::try_from(size).unwrap(),
                        });
                    let mut dst_group_offset = 0;
                    out_ptr.add(4usize).cast::<u32>().write_unaligned(self.hid);
                    out_ptr.add(8usize).cast::<u32>().write_unaligned(self.uid);
                    out_ptr.add(12usize).cast::<u32>().write_unaligned(self.cid);
                    self.hid_str.as_ptr().cast::<u8>().copy_to_nonoverlapping(
                        out_ptr.add(16usize + dst_group_offset),
                        size_of_val(self.hid_str),
                    );
                    dst_group_offset += size_of_val(self.hid_str);
                    self.uid_str.as_ptr().cast::<u8>().copy_to_nonoverlapping(
                        out_ptr.add(16usize + dst_group_offset),
                        size_of_val(self.uid_str),
                    );
                    dst_group_offset += size_of_val(self.uid_str);
                    self.cid_str.as_ptr().cast::<u8>().copy_to_nonoverlapping(
                        out_ptr.add(16usize + dst_group_offset),
                        size_of_val(self.cid_str),
                    );
                }
            }
        }

        /// ADR ACPI device path node.
        pub struct Adr<'a> {
            /// ADR values. For video output devices the value of this field
            /// comes from Table B-2 ACPI 3.0 specification. At least one
            /// ADR value is required.
            pub adr: &'a AdrSlice,
        }

        unsafe impl<'a> BuildNode for Adr<'a> {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 4usize + size_of_val(self.adr);
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::ACPI,
                            sub_type: DeviceSubType::ACPI_ADR,
                            length: u16::try_from(size).unwrap(),
                        });
                    self.adr
                        .as_ptr()
                        .cast::<u8>()
                        .copy_to_nonoverlapping(out_ptr.add(4usize), size_of_val(self.adr));
                }
            }
        }

        /// NVDIMM ACPI device path node.
        pub struct Nvdimm {
            /// NFIT device handle.
            pub nfit_device_handle: u32,
        }

        unsafe impl BuildNode for Nvdimm {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 8usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::ACPI,
                            sub_type: DeviceSubType::ACPI_NVDIMM,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<u32>()
                        .write_unaligned(self.nfit_device_handle);
                }
            }
        }

        /// Wrapper for [`u32`] ADR values that enforces at least one
        /// element is present.
        #[repr(transparent)]
        pub struct AdrSlice([u32]);
        impl AdrSlice {
            /// Create a new `AdrSlice`. Returns `None` if the input slice
            /// is empty.
            pub fn new(slice: &[u32]) -> Option<&Self> {
                if slice.is_empty() {
                    None
                } else {
                    let adr_slice: &Self = unsafe { core::mem::transmute(slice) };
                    Some(adr_slice)
                }
            }

            fn as_ptr(&self) -> *const u32 {
                self.0.as_ptr()
            }
        }
    }

    /// Device path build nodes for [`DeviceType::MESSAGING`].
    pub mod messaging {
        use super::*;
        /// ATAPI messaging device path node.
        pub struct Atapi {
            /// Whether the ATAPI device is primary or secondary.
            pub primary_secondary: crate::proto::device_path::messaging::PrimarySecondary,
            /// Whether the ATAPI device is master or slave.
            pub master_slave: crate::proto::device_path::messaging::MasterSlave,
            /// Logical Unit Number (LUN).
            pub logical_unit_number: u16,
        }

        unsafe impl BuildNode for Atapi {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 8usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_ATAPI,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<crate::proto::device_path::messaging::PrimarySecondary>()
                        .write_unaligned(self.primary_secondary);
                    out_ptr
                        .add(5usize)
                        .cast::<crate::proto::device_path::messaging::MasterSlave>()
                        .write_unaligned(self.master_slave);
                    out_ptr
                        .add(6usize)
                        .cast::<u16>()
                        .write_unaligned(self.logical_unit_number);
                }
            }
        }

        /// SCSI messaging device path node.
        pub struct Scsi {
            /// Target ID on the SCSI bus.
            pub target_id: u16,
            /// Logical Unit Number.
            pub logical_unit_number: u16,
        }

        unsafe impl BuildNode for Scsi {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 8usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_SCSI,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<u16>()
                        .write_unaligned(self.target_id);
                    out_ptr
                        .add(6usize)
                        .cast::<u16>()
                        .write_unaligned(self.logical_unit_number);
                }
            }
        }

        /// Fibre channel messaging device path node.
        pub struct FibreChannel {
            /// Fibre Channel World Wide Name.
            pub world_wide_name: u64,
            /// Fibre Channel Logical Unit Number.
            pub logical_unit_number: u64,
        }

        unsafe impl BuildNode for FibreChannel {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 24usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_FIBRE_CHANNEL,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr.add(4usize).write_bytes(0, size_of::<u32>());
                    out_ptr
                        .add(8usize)
                        .cast::<u64>()
                        .write_unaligned(self.world_wide_name);
                    out_ptr
                        .add(16usize)
                        .cast::<u64>()
                        .write_unaligned(self.logical_unit_number);
                }
            }
        }

        /// Fibre channel extended messaging device path node.
        pub struct FibreChannelEx {
            /// Fibre Channel end device port name (aka World Wide Name).
            pub world_wide_name: [u8; 8usize],
            /// Fibre Channel Logical Unit Number.
            pub logical_unit_number: [u8; 8usize],
        }

        unsafe impl BuildNode for FibreChannelEx {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 24usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_FIBRE_CHANNEL_EX,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr.add(4usize).write_bytes(0, size_of::<u32>());
                    out_ptr
                        .add(8usize)
                        .cast::<[u8; 8usize]>()
                        .write_unaligned(self.world_wide_name);
                    out_ptr
                        .add(16usize)
                        .cast::<[u8; 8usize]>()
                        .write_unaligned(self.logical_unit_number);
                }
            }
        }

        /// 1394 messaging device path node.
        pub struct Ieee1394 {
            /// 1394 Global Unique ID. Note that this is not the same as a
            /// UEFI GUID.
            pub guid: [u8; 8usize],
        }

        unsafe impl BuildNode for Ieee1394 {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 16usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_1394,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr.add(4usize).write_bytes(0, size_of::<u32>());
                    out_ptr
                        .add(8usize)
                        .cast::<[u8; 8usize]>()
                        .write_unaligned(self.guid);
                }
            }
        }

        /// USB messaging device path node.
        pub struct Usb {
            /// USB parent port number.
            pub parent_port_number: u8,
            /// USB interface number.
            pub interface: u8,
        }

        unsafe impl BuildNode for Usb {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 6usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_USB,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<u8>()
                        .write_unaligned(self.parent_port_number);
                    out_ptr
                        .add(5usize)
                        .cast::<u8>()
                        .write_unaligned(self.interface);
                }
            }
        }

        /// SATA messaging device path node.
        pub struct Sata {
            /// The HBA port number that facilitates the connection to the
            /// device or a port multiplier. The value 0xffff is reserved.
            pub hba_port_number: u16,
            /// the port multiplier port number that facilitates the
            /// connection to the device. Must be set to 0xffff if the
            /// device is directly connected to the HBA.
            pub port_multiplier_port_number: u16,
            /// Logical unit number.
            pub logical_unit_number: u16,
        }

        unsafe impl BuildNode for Sata {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 10usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_SATA,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<u16>()
                        .write_unaligned(self.hba_port_number);
                    out_ptr
                        .add(6usize)
                        .cast::<u16>()
                        .write_unaligned(self.port_multiplier_port_number);
                    out_ptr
                        .add(8usize)
                        .cast::<u16>()
                        .write_unaligned(self.logical_unit_number);
                }
            }
        }

        /// USB World Wide ID (WWID) messaging device path node.
        pub struct UsbWwid<'a> {
            /// USB interface number.
            pub interface_number: u16,
            /// USB vendor ID.
            pub device_vendor_id: u16,
            /// USB product ID.
            pub device_product_id: u16,
            /// Last 64 (or fewer) characters of the USB Serial number.
            pub serial_number: &'a [u16],
        }

        unsafe impl<'a> BuildNode for UsbWwid<'a> {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 10usize + size_of_val(self.serial_number);
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_USB_WWID,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<u16>()
                        .write_unaligned(self.interface_number);
                    out_ptr
                        .add(6usize)
                        .cast::<u16>()
                        .write_unaligned(self.device_vendor_id);
                    out_ptr
                        .add(8usize)
                        .cast::<u16>()
                        .write_unaligned(self.device_product_id);
                    self.serial_number
                        .as_ptr()
                        .cast::<u8>()
                        .copy_to_nonoverlapping(
                            out_ptr.add(10usize),
                            size_of_val(self.serial_number),
                        );
                }
            }
        }

        /// Device logical unit messaging device path node.
        pub struct DeviceLogicalUnit {
            /// Logical Unit Number.
            pub logical_unit_number: u8,
        }

        unsafe impl BuildNode for DeviceLogicalUnit {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 5usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_DEVICE_LOGICAL_UNIT,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<u8>()
                        .write_unaligned(self.logical_unit_number);
                }
            }
        }

        /// USB class messaging device path node.
        pub struct UsbClass {
            /// USB vendor ID.
            pub vendor_id: u16,
            /// USB product ID.
            pub product_id: u16,
            /// USB device class.
            pub device_class: u8,
            /// USB device subclass.
            pub device_subclass: u8,
            /// USB device protocol.
            pub device_protocol: u8,
        }

        unsafe impl BuildNode for UsbClass {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 11usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_USB_CLASS,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<u16>()
                        .write_unaligned(self.vendor_id);
                    out_ptr
                        .add(6usize)
                        .cast::<u16>()
                        .write_unaligned(self.product_id);
                    out_ptr
                        .add(8usize)
                        .cast::<u8>()
                        .write_unaligned(self.device_class);
                    out_ptr
                        .add(9usize)
                        .cast::<u8>()
                        .write_unaligned(self.device_subclass);
                    out_ptr
                        .add(10usize)
                        .cast::<u8>()
                        .write_unaligned(self.device_protocol);
                }
            }
        }

        /// I2O messaging device path node.
        pub struct I2o {
            /// Target ID (TID).
            pub target_id: u32,
        }

        unsafe impl BuildNode for I2o {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 8usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_I2O,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<u32>()
                        .write_unaligned(self.target_id);
                }
            }
        }

        /// MAC address messaging device path node.
        pub struct MacAddress {
            /// MAC address for a network interface, padded with zeros.
            pub mac_address: [u8; 32usize],
            /// Network interface type. See
            /// <https://www.iana.org/assignments/smi-numbers/smi-numbers.xhtml#smi-numbers-5>
            pub interface_type: u8,
        }

        unsafe impl BuildNode for MacAddress {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 37usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_MAC_ADDRESS,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<[u8; 32usize]>()
                        .write_unaligned(self.mac_address);
                    out_ptr
                        .add(36usize)
                        .cast::<u8>()
                        .write_unaligned(self.interface_type);
                }
            }
        }

        /// IPv4 messaging device path node.
        pub struct Ipv4 {
            /// Local IPv4 address.
            pub local_ip_address: [u8; 4usize],
            /// Remote IPv4 address.
            pub remote_ip_address: [u8; 4usize],
            /// Local port number.
            pub local_port: u16,
            /// Remote port number.
            pub remote_port: u16,
            /// Network protocol. See
            /// <https://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml>
            pub protocol: u16,
            /// Whether the source IP address is static or assigned via DHCP.
            pub ip_address_origin: crate::proto::device_path::messaging::Ipv4AddressOrigin,
            /// Gateway IP address.
            pub gateway_ip_address: [u8; 4usize],
            /// Subnet mask.
            pub subnet_mask: [u8; 4usize],
        }

        unsafe impl BuildNode for Ipv4 {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 27usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_IPV4,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<[u8; 4usize]>()
                        .write_unaligned(self.local_ip_address);
                    out_ptr
                        .add(8usize)
                        .cast::<[u8; 4usize]>()
                        .write_unaligned(self.remote_ip_address);
                    out_ptr
                        .add(12usize)
                        .cast::<u16>()
                        .write_unaligned(self.local_port);
                    out_ptr
                        .add(14usize)
                        .cast::<u16>()
                        .write_unaligned(self.remote_port);
                    out_ptr
                        .add(16usize)
                        .cast::<u16>()
                        .write_unaligned(self.protocol);
                    out_ptr
                        .add(18usize)
                        .cast::<crate::proto::device_path::messaging::Ipv4AddressOrigin>()
                        .write_unaligned(self.ip_address_origin);
                    out_ptr
                        .add(19usize)
                        .cast::<[u8; 4usize]>()
                        .write_unaligned(self.gateway_ip_address);
                    out_ptr
                        .add(23usize)
                        .cast::<[u8; 4usize]>()
                        .write_unaligned(self.subnet_mask);
                }
            }
        }

        /// IPv6 messaging device path node.
        pub struct Ipv6 {
            /// Local Ipv6 address.
            pub local_ip_address: [u8; 16usize],
            /// Remote Ipv6 address.
            pub remote_ip_address: [u8; 16usize],
            /// Local port number.
            pub local_port: u16,
            /// Remote port number.
            pub remote_port: u16,
            /// Network protocol. See
            /// <https://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml>
            pub protocol: u16,
            /// Origin of the local IP address.
            pub ip_address_origin: crate::proto::device_path::messaging::Ipv6AddressOrigin,
            /// Prefix length.
            pub prefix_length: u8,
            /// Gateway IP address.
            pub gateway_ip_address: [u8; 16usize],
        }

        unsafe impl BuildNode for Ipv6 {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 60usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_IPV6,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<[u8; 16usize]>()
                        .write_unaligned(self.local_ip_address);
                    out_ptr
                        .add(20usize)
                        .cast::<[u8; 16usize]>()
                        .write_unaligned(self.remote_ip_address);
                    out_ptr
                        .add(36usize)
                        .cast::<u16>()
                        .write_unaligned(self.local_port);
                    out_ptr
                        .add(38usize)
                        .cast::<u16>()
                        .write_unaligned(self.remote_port);
                    out_ptr
                        .add(40usize)
                        .cast::<u16>()
                        .write_unaligned(self.protocol);
                    out_ptr
                        .add(42usize)
                        .cast::<crate::proto::device_path::messaging::Ipv6AddressOrigin>()
                        .write_unaligned(self.ip_address_origin);
                    out_ptr
                        .add(43usize)
                        .cast::<u8>()
                        .write_unaligned(self.prefix_length);
                    out_ptr
                        .add(44usize)
                        .cast::<[u8; 16usize]>()
                        .write_unaligned(self.gateway_ip_address);
                }
            }
        }

        /// VLAN messaging device path node.
        pub struct Vlan {
            /// VLAN identifier (0-4094).
            pub vlan_id: u16,
        }

        unsafe impl BuildNode for Vlan {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 6usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_VLAN,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<u16>()
                        .write_unaligned(self.vlan_id);
                }
            }
        }

        /// InfiniBand messaging device path node.
        pub struct Infiniband {
            /// Flags to identify/manage InfiniBand elements.
            pub resource_flags: crate::proto::device_path::messaging::InfinibandResourceFlags,
            /// 128-bit Global Identifier for remote fabric port. Note that
            /// this is not the same as a UEFI GUID.
            pub port_gid: [u8; 16usize],
            /// IOC GUID if bit 0 of `resource_flags` is unset, or Service
            /// ID if bit 0 of `resource_flags` is set.
            pub ioc_guid_or_service_id: u64,
            /// 64-bit persistent ID of remote IOC port.
            pub target_port_id: u64,
            /// 64-bit persistent ID of remote device..
            pub device_id: u64,
        }

        unsafe impl BuildNode for Infiniband {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 48usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_INFINIBAND,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<crate::proto::device_path::messaging::InfinibandResourceFlags>()
                        .write_unaligned(self.resource_flags);
                    out_ptr
                        .add(8usize)
                        .cast::<[u8; 16usize]>()
                        .write_unaligned(self.port_gid);
                    out_ptr
                        .add(24usize)
                        .cast::<u64>()
                        .write_unaligned(self.ioc_guid_or_service_id);
                    out_ptr
                        .add(32usize)
                        .cast::<u64>()
                        .write_unaligned(self.target_port_id);
                    out_ptr
                        .add(40usize)
                        .cast::<u64>()
                        .write_unaligned(self.device_id);
                }
            }
        }

        /// UART messaging device path node.
        pub struct Uart {
            /// Baud rate setting, or 0 to use the device's default.
            pub baud_rate: u64,
            /// Number of data bits, or 0 to use the device's default.
            pub data_bits: u8,
            /// Parity setting.
            pub parity: crate::proto::device_path::messaging::Parity,
            /// Number of stop bits.
            pub stop_bits: crate::proto::device_path::messaging::StopBits,
        }

        unsafe impl BuildNode for Uart {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 19usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_UART,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr.add(4usize).write_bytes(0, size_of::<u32>());
                    out_ptr
                        .add(8usize)
                        .cast::<u64>()
                        .write_unaligned(self.baud_rate);
                    out_ptr
                        .add(16usize)
                        .cast::<u8>()
                        .write_unaligned(self.data_bits);
                    out_ptr
                        .add(17usize)
                        .cast::<crate::proto::device_path::messaging::Parity>()
                        .write_unaligned(self.parity);
                    out_ptr
                        .add(18usize)
                        .cast::<crate::proto::device_path::messaging::StopBits>()
                        .write_unaligned(self.stop_bits);
                }
            }
        }

        /// Vendor-defined messaging device path node.
        pub struct Vendor<'a> {
            /// Vendor-assigned GUID that defines the data that follows.
            pub vendor_guid: Guid,
            /// Vendor-defined data.
            pub vendor_defined_data: &'a [u8],
        }

        unsafe impl<'a> BuildNode for Vendor<'a> {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 20usize + size_of_val(self.vendor_defined_data);
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_VENDOR,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<Guid>()
                        .write_unaligned(self.vendor_guid);
                    self.vendor_defined_data
                        .as_ptr()
                        .cast::<u8>()
                        .copy_to_nonoverlapping(
                            out_ptr.add(20usize),
                            size_of_val(self.vendor_defined_data),
                        );
                }
            }
        }

        /// Serial Attached SCSI (SAS) extended messaging device path node.
        pub struct SasEx {
            /// SAS address.
            pub sas_address: [u8; 8usize],
            /// Logical Unit Number.
            pub logical_unit_number: [u8; 8usize],
            /// Information about the device and its interconnect.
            pub info: u16,
            /// Relative Target Port (RTP).
            pub relative_target_port: u16,
        }

        unsafe impl BuildNode for SasEx {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 24usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_SCSI_SAS_EX,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<[u8; 8usize]>()
                        .write_unaligned(self.sas_address);
                    out_ptr
                        .add(12usize)
                        .cast::<[u8; 8usize]>()
                        .write_unaligned(self.logical_unit_number);
                    out_ptr
                        .add(20usize)
                        .cast::<u16>()
                        .write_unaligned(self.info);
                    out_ptr
                        .add(22usize)
                        .cast::<u16>()
                        .write_unaligned(self.relative_target_port);
                }
            }
        }

        /// iSCSI messaging device path node.
        pub struct Iscsi<'a> {
            /// Network protocol.
            pub protocol: crate::proto::device_path::messaging::IscsiProtocol,
            /// iSCSI login options (bitfield).
            pub options: crate::proto::device_path::messaging::IscsiLoginOptions,
            /// iSCSI Logical Unit Number.
            pub logical_unit_number: [u8; 8usize],
            /// iSCSI Target Portal group tag the initiator intends to
            /// establish a session with.
            pub target_portal_group_tag: u16,
            /// iSCSI Node Target name.
            ///
            /// The UEFI Specification does not specify how the string is
            /// encoded, but gives one example that appears to be
            /// null-terminated ASCII.
            pub iscsi_target_name: &'a [u8],
        }

        unsafe impl<'a> BuildNode for Iscsi<'a> {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 18usize + size_of_val(self.iscsi_target_name);
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_ISCSI,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<crate::proto::device_path::messaging::IscsiProtocol>()
                        .write_unaligned(self.protocol);
                    out_ptr
                        .add(6usize)
                        .cast::<crate::proto::device_path::messaging::IscsiLoginOptions>()
                        .write_unaligned(self.options);
                    out_ptr
                        .add(8usize)
                        .cast::<[u8; 8usize]>()
                        .write_unaligned(self.logical_unit_number);
                    out_ptr
                        .add(16usize)
                        .cast::<u16>()
                        .write_unaligned(self.target_portal_group_tag);
                    self.iscsi_target_name
                        .as_ptr()
                        .cast::<u8>()
                        .copy_to_nonoverlapping(
                            out_ptr.add(18usize),
                            size_of_val(self.iscsi_target_name),
                        );
                }
            }
        }

        /// NVM Express namespace messaging device path node.
        pub struct NvmeNamespace {
            /// Namespace identifier (NSID). The values 0 and 0xffff_ffff
            /// are invalid.
            pub namespace_identifier: u32,
            /// IEEE Extended Unique Identifier (EUI-64), or 0 if the device
            /// does not have a EUI-64.
            pub ieee_extended_unique_identifier: u64,
        }

        unsafe impl BuildNode for NvmeNamespace {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 16usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_NVME_NAMESPACE,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<u32>()
                        .write_unaligned(self.namespace_identifier);
                    out_ptr
                        .add(8usize)
                        .cast::<u64>()
                        .write_unaligned(self.ieee_extended_unique_identifier);
                }
            }
        }

        /// Uniform Resource Identifier (URI) messaging device path node.
        pub struct Uri<'a> {
            /// URI as defined by [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986).
            pub value: &'a [u8],
        }

        unsafe impl<'a> BuildNode for Uri<'a> {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 4usize + size_of_val(self.value);
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_URI,
                            length: u16::try_from(size).unwrap(),
                        });
                    self.value
                        .as_ptr()
                        .cast::<u8>()
                        .copy_to_nonoverlapping(out_ptr.add(4usize), size_of_val(self.value));
                }
            }
        }

        /// Universal Flash Storage (UFS) messaging device path node.
        pub struct Ufs {
            /// Target ID on the UFS interface (PUN).
            pub target_id: u8,
            /// Logical Unit Number (LUN).
            pub logical_unit_number: u8,
        }

        unsafe impl BuildNode for Ufs {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 6usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_UFS,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<u8>()
                        .write_unaligned(self.target_id);
                    out_ptr
                        .add(5usize)
                        .cast::<u8>()
                        .write_unaligned(self.logical_unit_number);
                }
            }
        }

        /// Secure Digital (SD) messaging device path node.
        pub struct Sd {
            /// Slot number.
            pub slot_number: u8,
        }

        unsafe impl BuildNode for Sd {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 5usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_SD,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<u8>()
                        .write_unaligned(self.slot_number);
                }
            }
        }

        /// Bluetooth messaging device path node.
        pub struct Bluetooth {
            /// 48-bit bluetooth device address.
            pub device_address: [u8; 6usize],
        }

        unsafe impl BuildNode for Bluetooth {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 10usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_BLUETOOTH,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<[u8; 6usize]>()
                        .write_unaligned(self.device_address);
                }
            }
        }

        /// Wi-Fi messaging device path node.
        pub struct Wifi {
            /// Service set identifier (SSID).
            pub ssid: [u8; 32usize],
        }

        unsafe impl BuildNode for Wifi {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 36usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_WIFI,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<[u8; 32usize]>()
                        .write_unaligned(self.ssid);
                }
            }
        }

        /// Embedded Multi-Media Card (eMMC) messaging device path node.
        pub struct Emmc {
            /// Slot number.
            pub slot_number: u8,
        }

        unsafe impl BuildNode for Emmc {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 5usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_EMMC,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<u8>()
                        .write_unaligned(self.slot_number);
                }
            }
        }

        /// BluetoothLE messaging device path node.
        pub struct BluetoothLe {
            /// 48-bit bluetooth device address.
            pub device_address: [u8; 6usize],
            /// Address type.
            pub address_type: crate::proto::device_path::messaging::BluetoothLeAddressType,
        }

        unsafe impl BuildNode for BluetoothLe {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 11usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_BLUETOOTH_LE,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<[u8; 6usize]>()
                        .write_unaligned(self.device_address);
                    out_ptr
                        .add(10usize)
                        .cast::<crate::proto::device_path::messaging::BluetoothLeAddressType>()
                        .write_unaligned(self.address_type);
                }
            }
        }

        /// DNS messaging device path node.
        pub struct Dns<'a> {
            /// Whether the addresses are IPv4 or IPv6.
            pub address_type: crate::proto::device_path::messaging::DnsAddressType,
            /// One or more instances of the DNS server address.
            pub addresses: &'a [IpAddress],
        }

        unsafe impl<'a> BuildNode for Dns<'a> {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 5usize + size_of_val(self.addresses);
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_DNS,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<crate::proto::device_path::messaging::DnsAddressType>()
                        .write_unaligned(self.address_type);
                    self.addresses
                        .as_ptr()
                        .cast::<u8>()
                        .copy_to_nonoverlapping(out_ptr.add(5usize), size_of_val(self.addresses));
                }
            }
        }

        /// NVDIMM namespace messaging device path node.
        pub struct NvdimmNamespace {
            /// Namespace unique label identifier.
            pub uuid: [u8; 16usize],
        }

        unsafe impl BuildNode for NvdimmNamespace {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 20usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_NVDIMM_NAMESPACE,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<[u8; 16usize]>()
                        .write_unaligned(self.uuid);
                }
            }
        }

        /// REST service messaging device path node.
        pub struct RestService<'a> {
            /// Type of REST service.
            pub service_type: crate::proto::device_path::messaging::RestServiceType,
            /// Whether the service is in-band or out-of-band.
            pub access_mode: crate::proto::device_path::messaging::RestServiceAccessMode,
            /// Vendor-specific data. Only used if the service type is [`VENDOR`].
            ///
            /// [`VENDOR`]: uefi::proto::device_path::messaging::RestServiceType
            pub vendor_guid_and_data: Option<RestServiceVendorData<'a>>,
        }

        unsafe impl<'a> BuildNode for RestService<'a> {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 6usize + self.build_size_vendor_guid_and_data();
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_REST_SERVICE,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<crate::proto::device_path::messaging::RestServiceType>()
                        .write_unaligned(self.service_type);
                    out_ptr
                        .add(5usize)
                        .cast::<crate::proto::device_path::messaging::RestServiceAccessMode>()
                        .write_unaligned(self.access_mode);
                    self.build_vendor_guid_and_data(&mut out[6usize..])
                }
            }
        }

        /// NVME over Fabric (NVMe-oF) namespace messaging device path node.
        pub struct NvmeOfNamespace<'a> {
            /// Namespace Identifier Type (NIDT).
            pub nidt: u8,
            /// Namespace Identifier (NID).
            pub nid: [u8; 16usize],
            /// Unique identifier of an NVM subsystem stored as a
            /// null-terminated UTF-8 string. Maximum length of 224 bytes.
            pub subsystem_nqn: &'a [u8],
        }

        unsafe impl<'a> BuildNode for NvmeOfNamespace<'a> {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 21usize + size_of_val(self.subsystem_nqn);
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MESSAGING,
                            sub_type: DeviceSubType::MESSAGING_NVME_OF_NAMESPACE,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr.add(4usize).cast::<u8>().write_unaligned(self.nidt);
                    out_ptr
                        .add(5usize)
                        .cast::<[u8; 16usize]>()
                        .write_unaligned(self.nid);
                    self.subsystem_nqn
                        .as_ptr()
                        .cast::<u8>()
                        .copy_to_nonoverlapping(
                            out_ptr.add(21usize),
                            size_of_val(self.subsystem_nqn),
                        );
                }
            }
        }

        /// Vendor-specific REST service data. Only used for service type [`VENDOR`].
        ///
        /// [`VENDOR`]: uefi::proto::device_path::messaging::RestServiceType
        pub struct RestServiceVendorData<'a> {
            /// Vendor GUID.
            pub vendor_guid: Guid,
            /// Vendor-defined data.
            pub vendor_defined_data: &'a [u8],
        }

        impl<'a> RestService<'a> {
            fn build_size_vendor_guid_and_data(&self) -> usize {
                if let Some(src) = &self.vendor_guid_and_data {
                    assert!(
                        self.service_type
                            == crate::proto::device_path::messaging::RestServiceType::VENDOR
                    );
                    size_of::<Guid>() + size_of_val(src.vendor_defined_data)
                } else {
                    0
                }
            }

            fn build_vendor_guid_and_data(&self, out: &mut [MaybeUninit<u8>]) {
                if let Some(src) = &self.vendor_guid_and_data {
                    assert!(
                        self.service_type
                            == crate::proto::device_path::messaging::RestServiceType::VENDOR
                    );
                    let (guid_out, data_out) = out.split_at_mut(size_of::<Guid>());
                    let guid_out_ptr: *mut Guid = MaybeUninit::slice_as_mut_ptr(guid_out).cast();
                    unsafe {
                        guid_out_ptr.write_unaligned(src.vendor_guid);
                    }

                    let data_out_ptr = MaybeUninit::slice_as_mut_ptr(data_out);
                    unsafe {
                        src.vendor_defined_data
                            .as_ptr()
                            .copy_to_nonoverlapping(data_out_ptr, data_out.len());
                    }
                }
            }
        }
    }

    /// Device path build nodes for [`DeviceType::MEDIA`].
    pub mod media {
        use super::*;
        /// Hard drive media device path node.
        pub struct HardDrive {
            /// Index of the partition, starting from 1.
            pub partition_number: u32,
            /// Starting LBA (logical block address) of the partition.
            pub partition_start: u64,
            /// Size of the partition in blocks.
            pub partition_size: u64,
            /// Partition signature.
            pub partition_signature: crate::proto::device_path::media::PartitionSignature,
            /// Partition format.
            pub partition_format: crate::proto::device_path::media::PartitionFormat,
        }

        unsafe impl BuildNode for HardDrive {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 42usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MEDIA,
                            sub_type: DeviceSubType::MEDIA_HARD_DRIVE,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<u32>()
                        .write_unaligned(self.partition_number);
                    out_ptr
                        .add(8usize)
                        .cast::<u64>()
                        .write_unaligned(self.partition_start);
                    out_ptr
                        .add(16usize)
                        .cast::<u64>()
                        .write_unaligned(self.partition_size);
                    out_ptr
                        .add(24usize)
                        .cast::<[u8; 16usize]>()
                        .write_unaligned(self.build_partition_signature());
                    out_ptr
                        .add(40usize)
                        .cast::<crate::proto::device_path::media::PartitionFormat>()
                        .write_unaligned(self.partition_format);
                    out_ptr
                        .add(41usize)
                        .cast::<u8>()
                        .write_unaligned(self.build_signature_type());
                }
            }
        }

        /// CD-ROM media device path node.
        pub struct CdRom {
            /// Boot entry number from the boot catalog, or 0 for the
            /// default entry.
            pub boot_entry: u32,
            /// Starting RBA (Relative logical Block Address).
            pub partition_start: u64,
            /// Size of the partition in blocks.
            pub partition_size: u64,
        }

        unsafe impl BuildNode for CdRom {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 24usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MEDIA,
                            sub_type: DeviceSubType::MEDIA_CD_ROM,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<u32>()
                        .write_unaligned(self.boot_entry);
                    out_ptr
                        .add(8usize)
                        .cast::<u64>()
                        .write_unaligned(self.partition_start);
                    out_ptr
                        .add(16usize)
                        .cast::<u64>()
                        .write_unaligned(self.partition_size);
                }
            }
        }

        /// Vendor-defined media device path node.
        pub struct Vendor<'a> {
            /// Vendor-assigned GUID that defines the data that follows.
            pub vendor_guid: Guid,
            /// Vendor-defined data.
            pub vendor_defined_data: &'a [u8],
        }

        unsafe impl<'a> BuildNode for Vendor<'a> {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 20usize + size_of_val(self.vendor_defined_data);
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MEDIA,
                            sub_type: DeviceSubType::MEDIA_VENDOR,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<Guid>()
                        .write_unaligned(self.vendor_guid);
                    self.vendor_defined_data
                        .as_ptr()
                        .cast::<u8>()
                        .copy_to_nonoverlapping(
                            out_ptr.add(20usize),
                            size_of_val(self.vendor_defined_data),
                        );
                }
            }
        }

        /// File path media device path node.
        pub struct FilePath<'a> {
            /// Null-terminated path.
            pub path_name: &'a CStr16,
        }

        unsafe impl<'a> BuildNode for FilePath<'a> {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 4usize + size_of_val(self.path_name);
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MEDIA,
                            sub_type: DeviceSubType::MEDIA_FILE_PATH,
                            length: u16::try_from(size).unwrap(),
                        });
                    self.path_name
                        .as_ptr()
                        .cast::<u8>()
                        .copy_to_nonoverlapping(out_ptr.add(4usize), size_of_val(self.path_name));
                }
            }
        }

        /// Media protocol media device path node.
        pub struct Protocol {
            /// The ID of the protocol.
            pub protocol_guid: Guid,
        }

        unsafe impl BuildNode for Protocol {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 20usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MEDIA,
                            sub_type: DeviceSubType::MEDIA_PROTOCOL,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<Guid>()
                        .write_unaligned(self.protocol_guid);
                }
            }
        }

        /// PIWG firmware file media device path node.
        pub struct PiwgFirmwareFile<'a> {
            /// Contents are defined in the UEFI PI Specification.
            pub data: &'a [u8],
        }

        unsafe impl<'a> BuildNode for PiwgFirmwareFile<'a> {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 4usize + size_of_val(self.data);
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MEDIA,
                            sub_type: DeviceSubType::MEDIA_PIWG_FIRMWARE_FILE,
                            length: u16::try_from(size).unwrap(),
                        });
                    self.data
                        .as_ptr()
                        .cast::<u8>()
                        .copy_to_nonoverlapping(out_ptr.add(4usize), size_of_val(self.data));
                }
            }
        }

        /// PIWG firmware volume media device path node.
        pub struct PiwgFirmwareVolume<'a> {
            /// Contents are defined in the UEFI PI Specification.
            pub data: &'a [u8],
        }

        unsafe impl<'a> BuildNode for PiwgFirmwareVolume<'a> {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 4usize + size_of_val(self.data);
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MEDIA,
                            sub_type: DeviceSubType::MEDIA_PIWG_FIRMWARE_VOLUME,
                            length: u16::try_from(size).unwrap(),
                        });
                    self.data
                        .as_ptr()
                        .cast::<u8>()
                        .copy_to_nonoverlapping(out_ptr.add(4usize), size_of_val(self.data));
                }
            }
        }

        /// Relative offset range media device path node.
        pub struct RelativeOffsetRange {
            /// Offset of the first byte, relative to the parent device node.
            pub starting_offset: u64,
            /// Offset of the last byte, relative to the parent device node.
            pub ending_offset: u64,
        }

        unsafe impl BuildNode for RelativeOffsetRange {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 24usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MEDIA,
                            sub_type: DeviceSubType::MEDIA_RELATIVE_OFFSET_RANGE,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr.add(4usize).write_bytes(0, size_of::<u32>());
                    out_ptr
                        .add(8usize)
                        .cast::<u64>()
                        .write_unaligned(self.starting_offset);
                    out_ptr
                        .add(16usize)
                        .cast::<u64>()
                        .write_unaligned(self.ending_offset);
                }
            }
        }

        /// RAM disk media device path node.
        pub struct RamDisk {
            /// Starting memory address.
            pub starting_address: u64,
            /// Ending memory address.
            pub ending_address: u64,
            /// Type of RAM disk.
            pub disk_type: crate::proto::device_path::media::RamDiskType,
            /// RAM disk instance number if supported, otherwise 0.
            pub disk_instance: u16,
        }

        unsafe impl BuildNode for RamDisk {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 38usize;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::MEDIA,
                            sub_type: DeviceSubType::MEDIA_RAM_DISK,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<u64>()
                        .write_unaligned(self.starting_address);
                    out_ptr
                        .add(12usize)
                        .cast::<u64>()
                        .write_unaligned(self.ending_address);
                    out_ptr
                        .add(20usize)
                        .cast::<crate::proto::device_path::media::RamDiskType>()
                        .write_unaligned(self.disk_type);
                    out_ptr
                        .add(36usize)
                        .cast::<u16>()
                        .write_unaligned(self.disk_instance);
                }
            }
        }

        impl HardDrive {
            fn build_partition_signature(&self) -> [u8; 16] {
                use crate::proto::device_path::media::PartitionSignature::*;
                match self.partition_signature {
                    None => [0u8; 16],
                    Mbr(mbr) => {
                        let mut sig = [0u8; 16];
                        sig[0..4].copy_from_slice(&mbr);
                        sig
                    }

                    Guid(guid) => guid.to_bytes(),
                    Unknown { signature, .. } => signature,
                }
            }

            fn build_signature_type(&self) -> u8 {
                use crate::proto::device_path::media::PartitionSignature::*;
                match self.partition_signature {
                    None => 0,
                    Mbr(_) => 1,
                    Guid(_) => 2,
                    Unknown { signature_type, .. } => signature_type,
                }
            }
        }
    }

    /// Device path build nodes for [`DeviceType::BIOS_BOOT_SPEC`].
    pub mod bios_boot_spec {
        use super::*;
        /// BIOS Boot Specification device path node.
        pub struct BootSpecification<'a> {
            /// Device type as defined by the BIOS Boot Specification.
            pub device_type: u16,
            /// Status flags as defined by the BIOS Boot Specification.
            pub status_flag: u16,
            /// Description of the boot device encoded as a null-terminated
            /// ASCII string.
            pub description_string: &'a [u8],
        }

        unsafe impl<'a> BuildNode for BootSpecification<'a> {
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = 8usize + size_of_val(self.description_string);
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }

            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                assert_eq!(size, out.len());
                let out_ptr: *mut u8 = MaybeUninit::slice_as_mut_ptr(out);
                unsafe {
                    out_ptr
                        .cast::<DevicePathHeader>()
                        .write_unaligned(DevicePathHeader {
                            device_type: DeviceType::BIOS_BOOT_SPEC,
                            sub_type: DeviceSubType::BIOS_BOOT_SPECIFICATION,
                            length: u16::try_from(size).unwrap(),
                        });
                    out_ptr
                        .add(4usize)
                        .cast::<u16>()
                        .write_unaligned(self.device_type);
                    out_ptr
                        .add(6usize)
                        .cast::<u16>()
                        .write_unaligned(self.status_flag);
                    self.description_string
                        .as_ptr()
                        .cast::<u8>()
                        .copy_to_nonoverlapping(
                            out_ptr.add(8usize),
                            size_of_val(self.description_string),
                        );
                }
            }
        }
    }
}
