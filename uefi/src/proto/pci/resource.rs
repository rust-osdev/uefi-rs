// SPDX-License-Identifier: MIT OR Apache-2.0

//! Defines additional features for [`QWordAddressSpaceDescriptor`].

use core::ops::RangeInclusive;
use uefi_raw::protocol::pci::resource::{QWordAddressSpaceDescriptor, ResourceType};

/// Describes resource type specific flags.
/// ACPI Specification:
/// https://uefi.org/htmlspecs/ACPI_Spec_6_4_html/06_Device_Configuration/Device_Configuration.html#type-specific-attributes
#[derive(Debug)]
pub enum TypeFlag {
    /// Flags for Memory type resource.
    Memory(MemoryFlag),
    /// Flags for Io type resource.
    Io(IoFlags),
    /// Flags for Bus type resource.
    Bus(BusFlags),
}

/// Flags for Memory type resource.
/// ACPI Specification:
/// https://uefi.org/htmlspecs/ACPI_Spec_6_4_html/06_Device_Configuration/Device_Configuration.html#memory-resource-flag-resource-type-0-definitions
#[derive(Debug)]
pub struct MemoryFlag {
    /// Specifies if this resource is I/O on primary side of the bridge.
    /// [`TranslationType::TRANSLATION]` means it's memory on secondary bridge, I/O on primary bridge.
    /// [`TranslationType::STATIC]` means it's memory on both primary and secondary bridge.
    pub translation_type: TranslationType,

    /// Specifies properties of address range from this resource.
    /// It's only defined when this resource describes system RAM.
    pub mtp_attribute: MtpAttribute,

    /// Specifies cache properties of this resource
    /// Note: OSPM ignores this field in the Extended address space descriptor.
    /// Instead, it uses the Type Specific Attributes field to determine memory attributes.
    pub mem_attribute: MemAttribute,

    /// Specifies write-ability of this resource.
    pub write_status: WriteStatus,
}

/// Flags for Io type resource.
/// ACP Specification:
/// https://uefi.org/htmlspecs/ACPI_Spec_6_4_html/06_Device_Configuration/Device_Configuration.html#io-resource-flag-resource-type-1-definitions
#[derive(Debug)]
pub struct IoFlags {
    /// Specifies sparsity of address translation.
    /// It's only meaningful when translation_type below is [`TranslationType::TRANSLATION`].
    pub translation_sparsity: TranslationSparsity,

    /// Specifies if this resource is I/O on primary side of the bridge.
    /// [`TranslationType::TRANSLATION]` means it's I/O on secondary bridge, memory on primary bridge.
    /// [`TranslationType::STATIC]` means it's I/O on both secondary and primary bridge.
    pub translation_type: TranslationType,

    /// Specifies window range of Rng.
    pub rng_range: RngRange,
}

/// Flags for Bus type resource.
/// Currently, it's unused and all bits should be 0.
/// ACPI Specification:
/// https://uefi.org/htmlspecs/ACPI_Spec_6_4_html/06_Device_Configuration/Device_Configuration.html#bus-number-range-resource-flag-resource-type-2-definitions
#[derive(Debug)]
pub struct BusFlags {
    _reserved: u8,
}

newtype_enum! {
    /// Defines translation type flag.
    pub enum TranslationType: u8 => {
        /// Type of this resource is different on primary side of the bridge.
        TRANSLATION = 1,

        /// Type of this resource is same on primary side of the bridge.
        STATIC = 0,
    }
}

newtype_enum! {
    /// Defines memory range attribute flag.
    pub enum MtpAttribute: u8 => {
        /// This range is available RAM usable by the operating system.
        MEMORY = 0x0,

        /// This range of addresses is in use or reserved by the system
        /// and is not to be included in the allocatable memory pool
        /// of the operating system’s memory manager.
        RESERVED = 0x1,

        /// ACPI Reclaim Memory.
        /// This range is available RAM usable by the OS after it reads the ACPI tables.
        ACPI = 0x2,

        /// ACPI NVS Memory.
        /// This range of addresses is in use or reserved by the system
        /// and must not be used by the operating system.
        /// This range is required to be saved and restored across an NVS sleep.
        NVS = 0x3,
    }
}

newtype_enum! {
    /// Defines memory cache attribute flag.
    pub enum MemAttribute: u8 => {
        /// Memory is non-cacheable.
        NON_CACHEABLE = 0x0,

        /// Memory is cacheable.
        CACHEABLE = 0x1,

        /// Memory is cacheable and supports write combining.
        WRITE_COMBINE = 0x2,

        /// The memory is cacheable and prefetchable.
        PREFETCH = 0x3,
    }
}

newtype_enum! {
    /// Defines write status flag.
    pub enum WriteStatus: u8 => {
        /// This memory range is read-write.
        READ_WRITE = 1,

        /// This memory range is read-only.
        READ_ONLY = 0,
    }
}

newtype_enum! {
    /// Defines address translation sparsity flag.
    pub enum TranslationSparsity: u8 => {
        /// The primary-side memory address of any specific I/O port within
        /// the secondary-side range can be found using the following function.
        /// address = (((port & 0xFFFc) << 10) || (port & 0xFFF)) + [`QWordAddressSpaceDescriptor#translation_offset`]
        /// In the address used to access the I/O port, bits[11:2] must be identical to bits[21:12],
        /// this gives four bytes of I/O ports on each 4 KB page.
        SPARSE = 1,

        /// The primary-side memory address of any specific I/O port within
        /// the secondary-side range can be found using the following function.
        /// address = port + [`QWordAddressSpaceDescriptor#translation_offset`]
        DENSE = 0,
    }
}

newtype_enum! {
    /// Defines rng window range flag.
    pub enum RngRange: u8 => {
        /// Memory window covers the entire range
        ALL = 3,

        /// ISARangesOnly.
        /// This flag is for bridges on systems with multiple bridges.
        /// Setting this bit means the memory window specified in this descriptor is
        /// limited to the ISA I/O addresses that fall within the specified window.
        /// The ISA I/O ranges are: n000-n0FF, n400-n4FF, n800-n8FF, nC00-nCFF.
        /// This bit can only be set for bridges entirely configured throughACPI namespace.
        ISA_ONLY = 2,

        /// NonISARangesOnly.
        /// This flag is for bridges on systems with multiple bridges.
        /// Setting this bit means the memory window specified in this descriptor is
        /// limited to the non-ISA I/O addresses that fall within the specified window.
        /// The non-ISA I/O ranges are: n100-n3FF, n500-n7FF, n900-nBFF, nD00-nFFF.
        /// This bit can only be set for bridges entirely configured through ACPI namespace.
        NON_ISA_ONLY = 1,
    }
}

/// Extension trait for [`QWordAddressSpaceDescriptor`].
pub trait QWordAddressSpaceDescriptorExt {
    /// Returns type-specific flags of this descriptor
    fn type_flags(&self) -> TypeFlag {
        match self.descriptor().resource_type {
            ResourceType::MEMORY => TypeFlag::Memory(MemoryFlag::new(self.descriptor().type_flags)),
            ResourceType::IO => TypeFlag::Io(IoFlags::new(self.descriptor().type_flags)),
            ResourceType::BUS => TypeFlag::Bus(BusFlags::new(self.descriptor().type_flags)),
            _ => unreachable!(),
        }
    }

    /// Returns if this descriptor is currently 64 bit or 32 bit.
    ///
    /// # Returns
    /// None: It's unspecified
    /// true: This descriptor is 64 bit
    /// false: This descriptor is 32 bit
    fn is_64bit(&self) -> Option<bool> {
        let granularity = self.descriptor().address_granularity;
        match granularity {
            32 => Some(false),
            64 => Some(true),
            _ => None,
        }
    }

    /// Returns address range of this descriptor.
    fn address_range(&self) -> RangeInclusive<u64> {
        let descriptor = self.descriptor();
        let offset_min = descriptor.range_min + descriptor.translation_offset;
        let offset_max = descriptor.range_max + descriptor.translation_offset;
        let length = descriptor.address_length;
        let range = offset_min..=offset_max;
        debug_assert_eq!(range.clone().count() as u64, length);
        range
    }

    #[allow(missing_docs)]
    fn descriptor(&self) -> &QWordAddressSpaceDescriptor;
}

impl QWordAddressSpaceDescriptorExt for QWordAddressSpaceDescriptor {
    fn descriptor(&self) -> &QWordAddressSpaceDescriptor {
        self
    }
}

impl MemoryFlag {
    /// Constructs new [`MemoryFlag`] from raw byte.
    ///
    /// # Panic
    /// Panics when reserved bits are not 0.
    pub fn new(flags: u8) -> Self {
        let write_status = WriteStatus(flags & 0b1);
        let mem_attribute = MemAttribute((flags >> 1) & 0b11);
        let mtp_attribute = MtpAttribute((flags >> 3) & 0b11);
        let translation_type = TranslationType((flags >> 5) & 0b1);
        assert_eq!((flags >> 6) & 0b11, 0b00);

        Self {
            translation_type,
            mtp_attribute,
            mem_attribute,
            write_status,
        }
    }
}

impl IoFlags {
    /// Constructs new [`IoFlags`] from raw byte.
    ///
    /// # Panic
    /// Panics when reserved bits are not 0.
    pub fn new(flags: u8) -> Self {
        assert_ne!(flags & 0b11, 0);
        let rng_range = RngRange(flags & 0b11);

        let translation_type = TranslationType((flags >> 4) & 0b1);
        let translation_sparsity = TranslationSparsity((flags >> 5) & 0b1);

        assert_eq!((flags >> 2) & 0b11, 0b00);
        assert_eq!((flags >> 6) & 0b11, 0b00);

        Self {
            translation_sparsity,
            translation_type,
            rng_range,
        }
    }
}

impl BusFlags {
    /// Constructs new [`BusFlags`] from raw byte.
    /// The byte must be 0.
    ///
    /// # Panic
    /// Panics when byte is not 0.
    pub fn new(flags: u8) -> Self {
        assert_eq!(flags, 0);
        Self { _reserved: 0 }
    }
}
