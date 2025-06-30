use bitflags::bitflags;
use static_assertions::assert_eq_size;

/// Descriptor for current PCI root bridge's configuration space.
/// Specification:
/// https://uefi.org/htmlspecs/ACPI_Spec_6_4_html/06_Device_Configuration/Device_Configuration.html#qword-address-space-descriptor
#[repr(C, packed)]
#[derive(Debug)]
pub struct QWordAddressSpaceDescriptor {
    pub tag: u8,
    pub descriptor_length: u16,
    pub resource_type: ResourceType,
    pub flags: GeneralFlags,
    pub type_flags: u8,
    pub address_granularity: u64,
    pub range_min: u64,
    pub range_max: u64, // inclusive
    pub translation_offset: u64,
    pub address_length: u64,
}
assert_eq_size!(QWordAddressSpaceDescriptor, [u8; 0x2E]);

newtype_enum! {
    /// Indicates which type of resource this descriptor describes.
    pub enum ResourceType: u8 => {
        /// This resource describes range of memory.
        MEMORY = 0,

        /// This resource describes range of I/O ports.
        IO = 1,

        /// This resource describes range of Bus numbers.
        BUS = 2,
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone)]
    pub struct GeneralFlags: u8 {
        /// Indicates maximum address is fixed.
        const MAX_ADDRESS_FIXED = 0b1000;

        /// Indicates minimum address is fixed.
        const MIN_ADDRESS_FIXED = 0b0100;

        /// Indicates if this bridge would subtract or positively decode address.
        /// 1 This bridge subtractively decodes this address (top level bridges only)
        /// 0 This bridge positively decodes this address
        const DECODE_TYPE = 0b0010;
    }
}

impl QWordAddressSpaceDescriptor {
    /// Verifies if given descriptor is valid according to specification.
    /// This also checks if all reserved bit fields which are supposed to be 0 are actually 0.
    pub fn verify(&self) {
        let tag = self.tag;
        if tag != 0x8A {
            panic!(
                "Tag value for QWordAddressSpaceDescriptor should be 0x8A, not {}",
                tag
            );
        }

        let length = self.descriptor_length;
        if self.descriptor_length != 0x2B {
            panic!(
                "Length value for QWordAddressSpaceDescriptor should be 0x2B, not {}",
                length
            );
        }

        if self.flags.bits() & 0b11110000 != 0 {
            panic!("Reserved bits for GeneralFlags are 1")
        }

        let type_flags = self.type_flags;
        match self.resource_type {
            ResourceType::MEMORY => {
                if type_flags & 0b11000000 != 0 {
                    panic!("Reserved bits for Memory Type Flags are 1");
                }
            }
            ResourceType::IO => {
                if type_flags & 0b11001100 != 0 {
                    panic!("Reserved bits for IO Type Flags are 1");
                }
            }
            ResourceType::BUS => {
                if type_flags != 0 {
                    panic!("Bus type flags should be 0, not {}", type_flags);
                }
            }
            ResourceType(3..=191) => panic!("Invalid resource type: {}", self.resource_type.0),
            ResourceType(192..) => {} // Hardware defined range
        }

        let min = self.range_min;
        let max = self.range_max;
        if max < min {
            panic!(
                "Address range is invalid. Max(0x{:X}) is smaller than Min(0x{:X}).",
                max, min
            );
        }
    }
}
