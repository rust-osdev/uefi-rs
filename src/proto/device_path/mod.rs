//! Device Path protocol
//!
//! A UEFI device path is a very flexible structure for encoding a
//! programatic path such as a hard drive or console.
//!
//! A device path is made up of a packed list of variable-length nodes of
//! various types. The entire device path is terminated with an
//! [`END_ENTIRE`] node. A device path _may_ contain multiple device-path
//! instances separated by [`END_INSTANCE`] nodes, but typical paths contain
//! only a single instance (in which case no `END_INSTANCE` node is needed).
//!
//! Example of what a device path containing two instances (each comprised of
//! three nodes) might look like:
//!
//! ```text
//! ┌──────┬─────┬──────────────╥───────┬──────────┬────────────┐
//! │ ACPI │ PCI │ END_INSTANCE ║ CDROM │ FILEPATH │ END_ENTIRE │
//! └──────┴─────┴──────────────╨───────┴──────────┴────────────┘
//! ↑                           ↑                               ↑
//! ├─── DevicePathInstance ────╨────── DevicePathInstance ─────┤
//! │                                                           │
//! └─────────────────── Entire DevicePath ─────────────────────┘
//! ```
//!
//! # Types
//!
//! To represent device paths, this module provides several types:
//!
//! * [`DevicePath`] is the root type that represents a full device
//!   path, containing one or more device path instance. It ends with an
//!   [`END_ENTIRE`] node. It implements [`Protocol`] (corresponding to
//!   `EFI_DEVICE_PATH_PROTOCOL`).
//!
//! * [`DevicePathInstance`] represents a single path instance within a
//!   device path. It ends with either an [`END_INSTANCE`] or [`END_ENTIRE`]
//!   node.
//!
//! * [`DevicePathNode`] represents a single node within a path. The
//!   node's [`device_type`] and [`sub_type`] must be examined to
//!   determine what type of data it contains.
//!
//!   Specific node types have their own structures, but only a few are
//!   currently implemented:
//!   * [`AcpiDevicePath`]
//!   * [`FilePathMediaDevicePath`]
//!   * [`HardDriveMediaDevicePath`]
//!
//! * [`DevicePathHeader`] is a header present at the start of every
//!   node. It describes the type of node as well as the node's size.
//!
//! * [`FfiDevicePath`] is an opaque type used whenever a device path
//!   pointer is passed to or from external UEFI interfaces (i.e. where
//!   the UEFI spec uses `const* EFI_DEVICE_PATH_PROTOCOL`, `*const
//!   FfiDevicePath` should be used in the Rust definition). Many of the
//!   other types in this module are DSTs, so pointers to the type are
//!   "fat" and not suitable for FFI.
//!
//! All of these types use a packed layout and may appear on any byte
//! boundary.
//!
//! Note: the API provided by this module is currently mostly limited to
//! reading existing device paths rather than constructing new ones.
//!
//! [`END_ENTIRE`]: DeviceSubType::END_ENTIRE
//! [`END_INSTANCE`]: DeviceSubType::END_INSTANCE
//! [`device_type`]: DevicePathNode::device_type
//! [`sub_type`]: DevicePathNode::sub_type

pub mod text;

use crate::data_types::UnalignedCStr16;
use crate::proto::{Protocol, ProtocolPointer};
use crate::{unsafe_guid, Guid};
use core::ffi::c_void;
use core::marker::{PhantomData, PhantomPinned};
use core::{mem, ptr};

/// Opaque type that should be used to represent a pointer to a
/// [`DevicePath`] or [`DevicePathNode`] in foreign function interfaces. This
/// type produces a thin pointer, unlike [`DevicePath`] and
/// [`DevicePathNode`].
#[repr(C, packed)]
pub struct FfiDevicePath {
    // This representation is recommended by the nomicon:
    // https://doc.rust-lang.org/stable/nomicon/ffi.html#representing-opaque-structs
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

/// Header that appears at the start of every [`DevicePathNode`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C, packed)]
pub struct DevicePathHeader {
    /// Type of device
    pub device_type: DeviceType,
    /// Sub type of device
    pub sub_type: DeviceSubType,
    /// Size (in bytes) of the [`DevicePathNode`], including this header.
    pub length: u16,
}

/// A single node within a [`DevicePath`].
///
/// Each node starts with a [`DevicePathHeader`]. The rest of the data
/// in the node depends on the type of node.
///
/// See the [module-level documentation] for more details.
///
/// [module-level documentation]: crate::proto::device_path
#[derive(Debug, Eq, PartialEq)]
#[repr(C, packed)]
pub struct DevicePathNode {
    header: DevicePathHeader,
    data: [u8],
}

impl DevicePathNode {
    /// Create a [`DevicePathNode`] reference from an opaque pointer.
    ///
    /// # Safety
    ///
    /// The input pointer must point to valid data. That data must
    /// remain valid for the lifetime `'a`, and cannot be mutated during
    /// that lifetime.
    pub unsafe fn from_ffi_ptr<'a>(ptr: *const FfiDevicePath) -> &'a DevicePathNode {
        let header = *ptr.cast::<DevicePathHeader>();

        let data_len = usize::from(header.length) - mem::size_of::<DevicePathHeader>();
        &*ptr::from_raw_parts(ptr.cast(), data_len)
    }

    /// Cast to a [`FfiDevicePath`] pointer.
    pub fn as_ffi_ptr(&self) -> *const FfiDevicePath {
        let ptr: *const Self = self;
        ptr.cast::<FfiDevicePath>()
    }

    /// Type of device
    pub fn device_type(&self) -> DeviceType {
        self.header.device_type
    }

    /// Sub type of device
    pub fn sub_type(&self) -> DeviceSubType {
        self.header.sub_type
    }

    /// Tuple of the node's type and subtype.
    pub fn full_type(&self) -> (DeviceType, DeviceSubType) {
        (self.header.device_type, self.header.sub_type)
    }

    /// Size (in bytes) of the full [`DevicePathNode`], including the header.
    pub fn length(&self) -> u16 {
        self.header.length
    }

    /// True if this node ends an entire [`DevicePath`].
    pub fn is_end_entire(&self) -> bool {
        self.full_type() == (DeviceType::END, DeviceSubType::END_ENTIRE)
    }

    /// Convert to a [`FilePathMediaDevicePath`]. Returns `None` if the
    /// node is not of the appropriate type.
    pub fn as_file_path_media_device_path(&self) -> Option<&FilePathMediaDevicePath> {
        if self.full_type() == (DeviceType::MEDIA, DeviceSubType::MEDIA_FILE_PATH) {
            // Get the length of the `path_name` field.
            let path_name_size_in_bytes = usize::from(self.header.length) - 4;
            assert!(path_name_size_in_bytes % mem::size_of::<u16>() == 0);
            let path_name_len = path_name_size_in_bytes / mem::size_of::<u16>();

            // Convert the `self` pointer to `FilePathMediaDevicePath`,
            // which is a DST as it ends in a slice.
            let p = self as *const Self;
            let p: *const FilePathMediaDevicePath = ptr::from_raw_parts(p.cast(), path_name_len);
            Some(unsafe { &*p })
        } else {
            None
        }
    }

    /// Convert to a [`HardDriveMediaDevicePath`]. Returns `None` if the
    /// node is not of the appropriate type.
    pub fn as_hard_drive_media_device_path(&self) -> Option<&HardDriveMediaDevicePath> {
        if self.full_type() == (DeviceType::MEDIA, DeviceSubType::MEDIA_HARD_DRIVE) {
            assert!({ self.header.length } == HARD_DRIVE_MEDIA_DEVICE_PATH_LENGTH);

            let p = self as *const Self;
            let p = p.cast::<HardDriveMediaDevicePath>();
            Some(unsafe { &*p })
        } else {
            None
        }
    }
}

/// A single device path instance that ends with either an [`END_INSTANCE`]
/// or [`END_ENTIRE`] node. Use [`DevicePath::instance_iter`] to get the
/// path instances in a [`DevicePath`].
///
/// See the [module-level documentation] for more details.
///
/// [`END_ENTIRE`]: DeviceSubType::END_ENTIRE
/// [`END_INSTANCE`]: DeviceSubType::END_INSTANCE
/// [module-level documentation]: crate::proto::device_path
#[repr(C, packed)]
#[derive(Debug, Eq, PartialEq)]
pub struct DevicePathInstance {
    data: [u8],
}

impl DevicePathInstance {
    /// Get an iterator over the [`DevicePathNodes`] in this
    /// instance. Iteration ends when any [`DeviceType::END`] node is
    /// reached.
    ///
    /// [`DevicePathNodes`]: DevicePathNode
    pub fn node_iter(&self) -> DevicePathNodeIterator {
        DevicePathNodeIterator {
            nodes: &self.data,
            stop_condition: StopCondition::AnyEndNode,
        }
    }
}

/// Device path protocol.
///
/// A device path contains one or more device path instances made of up
/// variable-length nodes. It ends with an [`END_ENTIRE`] node.
///
/// See the [module-level documentation] for more details.
///
/// [module-level documentation]: crate::proto::device_path
/// [`END_ENTIRE`]: DeviceSubType::END_ENTIRE
#[repr(C, packed)]
#[unsafe_guid("09576e91-6d3f-11d2-8e39-00a0c969723b")]
#[derive(Debug, Eq, PartialEq, Protocol)]
pub struct DevicePath {
    data: [u8],
}

impl ProtocolPointer for DevicePath {
    unsafe fn ptr_from_ffi(ptr: *const c_void) -> *const Self {
        ptr::from_raw_parts(ptr.cast(), Self::size_in_bytes_from_ptr(ptr))
    }

    unsafe fn mut_ptr_from_ffi(ptr: *mut c_void) -> *mut Self {
        ptr::from_raw_parts_mut(ptr.cast(), Self::size_in_bytes_from_ptr(ptr))
    }
}

impl DevicePath {
    /// Calculate the size in bytes of the entire `DevicePath` starting
    /// at `ptr`. This adds up each node's length, including the
    /// end-entire node.
    unsafe fn size_in_bytes_from_ptr(ptr: *const c_void) -> usize {
        let mut ptr = ptr.cast::<u8>();
        let mut total_size_in_bytes: usize = 0;
        loop {
            let node = DevicePathNode::from_ffi_ptr(ptr.cast::<FfiDevicePath>());
            let node_size_in_bytes = usize::from(node.length());
            total_size_in_bytes += node_size_in_bytes;
            if node.is_end_entire() {
                break;
            }
            ptr = ptr.add(node_size_in_bytes);
        }

        total_size_in_bytes
    }

    /// Create a [`DevicePath`] reference from an opaque pointer.
    ///
    /// # Safety
    ///
    /// The input pointer must point to valid data. That data must
    /// remain valid for the lifetime `'a`, and cannot be mutated during
    /// that lifetime.
    pub unsafe fn from_ffi_ptr<'a>(ptr: *const FfiDevicePath) -> &'a DevicePath {
        &*Self::ptr_from_ffi(ptr.cast::<c_void>())
    }

    /// Cast to a [`FfiDevicePath`] pointer.
    pub fn as_ffi_ptr(&self) -> *const FfiDevicePath {
        let p = self as *const Self;
        p.cast()
    }

    /// Get an iterator over the [`DevicePathInstance`]s in this path.
    pub fn instance_iter(&self) -> DevicePathInstanceIterator {
        DevicePathInstanceIterator {
            remaining_path: Some(self),
        }
    }

    /// Get an iterator over the [`DevicePathNode`]s starting at
    /// `self`. Iteration ends when a path is reached where
    /// [`is_end_entire`][DevicePathNode::is_end_entire] is true. That ending
    /// path is not returned by the iterator.
    pub fn node_iter(&self) -> DevicePathNodeIterator {
        DevicePathNodeIterator {
            nodes: &self.data,
            stop_condition: StopCondition::EndEntireNode,
        }
    }
}

/// Iterator over the [`DevicePathInstance`]s in a [`DevicePath`].
///
/// This struct is returned by [`DevicePath::instance_iter`].
pub struct DevicePathInstanceIterator<'a> {
    remaining_path: Option<&'a DevicePath>,
}

impl<'a> Iterator for DevicePathInstanceIterator<'a> {
    type Item = &'a DevicePathInstance;

    fn next(&mut self) -> Option<Self::Item> {
        let remaining_path = self.remaining_path?;

        let mut instance_size: usize = 0;

        // Find the end of the instance, which can be either kind of end
        // node (end-instance or end-entire). Count the number of bytes
        // up to and including that end node.
        let node_iter = DevicePathNodeIterator {
            nodes: &remaining_path.data,
            stop_condition: StopCondition::NoMoreNodes,
        };
        for node in node_iter {
            instance_size += usize::from(node.length());
            if node.device_type() == DeviceType::END {
                break;
            }
        }

        let (head, rest) = remaining_path.data.split_at(instance_size);

        if rest.is_empty() {
            self.remaining_path = None;
        } else {
            self.remaining_path = unsafe {
                Some(&*ptr::from_raw_parts(
                    rest.as_ptr().cast::<()>(),
                    rest.len(),
                ))
            };
        }

        unsafe {
            Some(&*ptr::from_raw_parts(
                head.as_ptr().cast::<()>(),
                head.len(),
            ))
        }
    }
}

enum StopCondition {
    AnyEndNode,
    EndEntireNode,
    NoMoreNodes,
}

/// Iterator over [`DevicePathNode`]s.
///
/// This struct is returned by [`DevicePath::node_iter`] and
/// [`DevicePathInstance::node_iter`].
pub struct DevicePathNodeIterator<'a> {
    nodes: &'a [u8],
    stop_condition: StopCondition,
}

impl<'a> Iterator for DevicePathNodeIterator<'a> {
    type Item = &'a DevicePathNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.nodes.is_empty() {
            return None;
        }

        let node =
            unsafe { DevicePathNode::from_ffi_ptr(self.nodes.as_ptr().cast::<FfiDevicePath>()) };

        // Check if an early stop condition has been reached.
        let stop = match self.stop_condition {
            StopCondition::AnyEndNode => node.device_type() == DeviceType::END,
            StopCondition::EndEntireNode => node.is_end_entire(),
            StopCondition::NoMoreNodes => false,
        };

        if stop {
            // Clear the remaining node data so that future calls to
            // next() immediately return `None`.
            self.nodes = &[];
            None
        } else {
            // Advance to next node.
            let node_size = usize::from(node.length());
            self.nodes = &self.nodes[node_size..];
            Some(node)
        }
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

/// File Path Media Device Path.
#[repr(C, packed)]
pub struct FilePathMediaDevicePath {
    header: DevicePathHeader,
    path_name: [u16],
}

impl FilePathMediaDevicePath {
    /// Get the path. An [`UnalignedCStr16`] is returned since this is a
    /// packed struct.
    pub fn path_name(&self) -> UnalignedCStr16<'_> {
        // Safety: creating this `UnalignedCStr16` is safe because the
        // `path_name` pointer is valid (although potentially
        // unaligned), and the lifetime of the output is tied to `self`,
        // so there's no possibility of use-after-free.
        unsafe {
            // Use `addr_of` to avoid creating an unaligned reference.
            let ptr: *const [u16] = ptr::addr_of!(self.path_name);
            let (ptr, len): (*const (), usize) = ptr.to_raw_parts();
            UnalignedCStr16::new(self, ptr.cast::<u16>(), len)
        }
    }
}

/// Hard Drive Media Device Path.
#[repr(C, packed)]
pub struct HardDriveMediaDevicePath {
    header: DevicePathHeader,
    partition_number: u32,
    partition_start: u64,
    partition_size: u64,
    partition_signature: PartitionSignatureUnion,
    partition_format: PartitionFormat,
    signature_type: SignatureType,
}

/// [`HardDriveMediaDevicePath`] is a fixed-length structure of 42 bytes.
const HARD_DRIVE_MEDIA_DEVICE_PATH_LENGTH: u16 = 42;

impl HardDriveMediaDevicePath {
    /// Returns the format of the partition (MBR, GPT, or unknown).
    pub fn partition_format(&self) -> PartitionFormat {
        self.partition_format
    }

    /// Returns the 1-based index of the partition.
    pub fn partition_number(&self) -> u32 {
        self.partition_number
    }

    /// Returns the partition size in logical blocks.
    pub fn partition_size(&self) -> u64 {
        self.partition_size
    }

    /// Returns the starting LBA of the partition.
    pub fn partition_start(&self) -> u64 {
        self.partition_start
    }

    /// Returns the MBR or GPT partition signature
    pub fn partition_signature(&self) -> Option<PartitionSignature> {
        match self.signature_type {
            SignatureType::MBR => {
                let mbr_signature = unsafe { self.partition_signature.mbr_signature };
                Some(PartitionSignature::MBR(mbr_signature))
            }
            SignatureType::GUID => {
                let guid = unsafe { self.partition_signature.guid };
                Some(PartitionSignature::GUID(guid))
            }
            _ => None,
        }
    }
}

newtype_enum! {
    /// Partition format.
    pub enum PartitionFormat: u8 => {
        /// MBR (PC-AT compatible Master Boot Record) format.
        MBR = 0x01,
        /// GPT (GUID Partition Table) format.
        GPT = 0x02,
    }
}

/// Partition signature.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PartitionSignature {
    /// 32-bit MBR partition signature.
    MBR(u32),
    /// 128-bit GUID partition signature.
    GUID(Guid),
}

#[repr(C)]
union PartitionSignatureUnion {
    mbr_signature: u32,
    guid: Guid,
}

newtype_enum! {
    /// Signature type.
    enum SignatureType: u8 => {
        /// 32-bit MBR partition signature.
        MBR = 0x01,
        /// 128-bit GUID partition signature.
        GUID = 0x02,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CString16;
    use alloc_api::vec::Vec;

    /// Create a node to `path` from raw data.
    fn add_node(path: &mut Vec<u8>, device_type: u8, sub_type: u8, node_data: &[u8]) {
        path.push(device_type);
        path.push(sub_type);
        path.extend(
            u16::try_from(mem::size_of::<DevicePathHeader>() + node_data.len())
                .unwrap()
                .to_le_bytes(),
        );
        path.extend(node_data);
    }

    /// Create a test device path list as raw bytes.
    fn create_raw_device_path() -> Vec<u8> {
        let mut raw_data = Vec::new();

        // First path instance.
        add_node(&mut raw_data, 0xa0, 0xb0, &[10, 11]);
        add_node(&mut raw_data, 0xa1, 0xb1, &[20, 21, 22, 23]);
        add_node(
            &mut raw_data,
            DeviceType::END.0,
            DeviceSubType::END_INSTANCE.0,
            &[],
        );
        // Second path instance.
        add_node(&mut raw_data, 0xa2, 0xb2, &[30, 31]);
        add_node(&mut raw_data, 0xa3, 0xb3, &[40, 41, 42, 43]);
        add_node(
            &mut raw_data,
            DeviceType::END.0,
            DeviceSubType::END_ENTIRE.0,
            &[],
        );

        raw_data
    }

    /// Check that `node` has the expected content.    
    fn check_node(node: &DevicePathNode, device_type: u8, sub_type: u8, node_data: &[u8]) {
        assert_eq!(node.device_type().0, device_type);
        assert_eq!(node.sub_type().0, sub_type);
        assert_eq!(
            node.length(),
            u16::try_from(mem::size_of::<DevicePathHeader>() + node_data.len()).unwrap()
        );
        assert_eq!(&node.data, node_data);
    }

    #[test]
    fn test_device_path_nodes() {
        let raw_data = create_raw_device_path();
        let dp = unsafe { DevicePath::from_ffi_ptr(raw_data.as_ptr().cast()) };

        // Check that the size is the sum of the nodes' lengths.
        assert_eq!(mem::size_of_val(dp), 6 + 8 + 4 + 6 + 8 + 4);

        // Check the list's node iter.
        let nodes: Vec<_> = dp.node_iter().collect();
        check_node(nodes[0], 0xa0, 0xb0, &[10, 11]);
        check_node(nodes[1], 0xa1, 0xb1, &[20, 21, 22, 23]);
        check_node(
            nodes[2],
            DeviceType::END.0,
            DeviceSubType::END_INSTANCE.0,
            &[],
        );
        check_node(nodes[3], 0xa2, 0xb2, &[30, 31]);
        check_node(nodes[4], 0xa3, 0xb3, &[40, 41, 42, 43]);
        // The end-entire node is not returned by the iterator.
        assert_eq!(nodes.len(), 5);
    }

    #[test]
    fn test_device_path_instances() {
        let raw_data = create_raw_device_path();
        let dp = unsafe { DevicePath::from_ffi_ptr(raw_data.as_ptr().cast()) };

        // Check the list's instance iter.
        let mut iter = dp.instance_iter();
        let mut instance = iter.next().unwrap();
        assert_eq!(mem::size_of_val(instance), 6 + 8 + 4);

        // Check the first instance's node iter.
        let nodes: Vec<_> = instance.node_iter().collect();
        check_node(nodes[0], 0xa0, 0xb0, &[10, 11]);
        check_node(nodes[1], 0xa1, 0xb1, &[20, 21, 22, 23]);
        // The end node is not returned by the iterator.
        assert_eq!(nodes.len(), 2);

        // Check second instance.
        instance = iter.next().unwrap();
        assert_eq!(mem::size_of_val(instance), 6 + 8 + 4);

        let nodes: Vec<_> = instance.node_iter().collect();
        check_node(nodes[0], 0xa2, 0xb2, &[30, 31]);
        check_node(nodes[1], 0xa3, 0xb3, &[40, 41, 42, 43]);
        // The end node is not returned by the iterator.
        assert_eq!(nodes.len(), 2);

        // Only two instances.
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_file_path_media() {
        // Manually create data for a `FilePathMediaDevicePath` node.
        let raw_data: [u16; 7] = [
            // MEDIA | MEDIA_FILE_PATH
            0x0404,
            // Length
            0x00_0e,
            b't'.into(),
            b'e'.into(),
            b's'.into(),
            b't'.into(),
            // Trailing null.
            0,
        ];

        // Convert the raw data to a `DevicePath` node.
        let dp = unsafe { DevicePathNode::from_ffi_ptr(raw_data.as_ptr().cast()) };
        assert_eq!(dp.length(), 14);

        // Check that the `file_name` is correct.
        let fpm = dp.as_file_path_media_device_path().unwrap();
        assert_eq!(
            fpm.path_name().to_cstring16().unwrap(),
            CString16::try_from("test").unwrap()
        );
    }

    #[test]
    fn test_hard_drive_media_mbr() {
        let mbr_partition_bytes: [u8; 42] = [
            0x04, 0x01, 0x2a, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3f, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xc1, 0xbf, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00, 0xfa, 0xfd, 0x1a, 0xbe,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01,
        ];
        let dp = unsafe { DevicePathNode::from_ffi_ptr(mbr_partition_bytes.as_ptr().cast()) };
        assert_eq!(dp.length(), HARD_DRIVE_MEDIA_DEVICE_PATH_LENGTH);

        let hdm = dp.as_hard_drive_media_device_path().unwrap();
        assert_eq!(hdm.partition_format(), PartitionFormat::MBR);
        assert_eq!(hdm.partition_number(), 1);
        assert_eq!(hdm.partition_size(), 1032129);
        assert_eq!(hdm.partition_start(), 63);
        assert_eq!(
            hdm.partition_signature(),
            Some(PartitionSignature::MBR(0xBE1AFDFA))
        );
    }

    #[test]
    fn test_hard_drive_media_gpt() {
        let guid_partition_bytes: [u8; 42] = [
            0x04, 0x01, 0x2a, 0x00, 0x01, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x10, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xa0, 0x39, 0xaa, 0x41,
            0x35, 0x3d, 0x84, 0x4f, 0xb1, 0x95, 0xae, 0x3a, 0x95, 0x0b, 0xfb, 0xad, 0x02, 0x02,
        ];
        let dp = unsafe { DevicePathNode::from_ffi_ptr(guid_partition_bytes.as_ptr().cast()) };
        assert_eq!(dp.length(), HARD_DRIVE_MEDIA_DEVICE_PATH_LENGTH);

        let hdm = dp.as_hard_drive_media_device_path().unwrap();
        assert_eq!(hdm.partition_format(), PartitionFormat::GPT);
        assert_eq!(hdm.partition_number(), 1);
        assert_eq!(hdm.partition_size(), 200704);
        assert_eq!(hdm.partition_start(), 128);
        assert_eq!(
            hdm.partition_signature(),
            Some(PartitionSignature::GUID(Guid::from_values(
                0x41aa39a0,
                0x3d35,
                0x4f84,
                0xb195,
                0xae3a950bfbad
            )))
        );
    }
}
