// SPDX-License-Identifier: MIT OR Apache-2.0

//! Pci root bus resource configuration descriptor parsing.

/// Represents the type of resource described by a QWORD Address Space Descriptor.
/// This corresponds to the `resource_type` field at offset 0x03 in the descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ResourceRangeType {
    /// Memory Range (value = 0)
    /// Indicates that the descriptor describes a memory-mapped address range.
    /// Commonly used for MMIO regions decoded by the PCI root bridge.
    Memory = 0,

    /// I/O Range (value = 1)
    /// Indicates that the descriptor describes a legacy I/O port range.
    /// Used for devices that communicate via port-mapped I/O.
    Io = 1,

    /// Bus Number Range (value = 2)
    /// Indicates that the descriptor describes a range of PCI bus numbers.
    /// Used to define the bus hierarchy behind a PCI root bridge.
    Bus = 2,

    /// Unknown or vendor-specific resource type.
    /// Captures any unrecognized value for forward compatibility.
    Unknown(u8),
}
impl From<u8> for ResourceRangeType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Memory,
            1 => Self::Io,
            2 => Self::Bus,
            other => Self::Unknown(other),
        }
    }
}

/// Represents a parsed QWORD Address Space Descriptor from UEFI.
/// This structure describes a decoded resource range for a PCI root bridge.
#[derive(Clone, Debug)]
pub struct QwordAddressSpaceDescriptor {
    /// Type of resource: Memory, I/O, Bus, or Unknown.
    pub resource_range_type: ResourceRangeType,
    /// General flags that describe decode behavior (e.g., positive decode).
    pub general_flags: u8,
    /// Type-specific flags (e.g., cacheability for memory).
    pub type_specific_flags: u8,
    /// Granularity of the address space (typically 32 or 64).
    /// Indicates whether the range is 32-bit or 64-bit.
    pub granularity: u64,
    /// Minimum address of the range (inclusive).
    pub address_min: u64,
    /// Maximum address of the range (inclusive).
    pub address_max: u64,
    /// Translation offset to convert host address to PCI address.
    /// Usually zero unless the bridge remaps addresses.
    pub translation_offset: u64,
    /// Length of the address range (in bytes or bus numbers).
    pub address_length: u64,
}

/// Parses a list of QWORD Address Space Descriptors from a raw memory region.
/// Stops when it encounters an End Tag descriptor (type 0x79).
#[cfg(feature = "alloc")]
pub(crate) fn parse(
    base: *const core::ffi::c_void,
) -> alloc::vec::Vec<QwordAddressSpaceDescriptor> {
    use alloc::slice;
    use alloc::vec::Vec;
    const PCI_RESTBL_QWORDADDRSPEC_TAG: u8 = 0x8a;
    const PCI_RESTBL_END_TAG: u8 = 0x79;

    let base: *const u8 = base.cast();

    // Phase 1: determine total length
    let mut offset = 0;
    loop {
        let tag = unsafe { core::ptr::read(base.add(offset)) };
        offset += match tag {
            PCI_RESTBL_QWORDADDRSPEC_TAG => 3 + 0x2B,
            PCI_RESTBL_END_TAG => break,
            _ => panic!("{tag}"), // Unknown tag - bailing
        };
    }

    // Phase 2: parse descriptors from resource table
    let mut bfr: &[u8] = unsafe { slice::from_raw_parts(base, offset) };
    let mut descriptors = Vec::new();
    while !bfr.is_empty() {
        match bfr[0] {
            PCI_RESTBL_QWORDADDRSPEC_TAG => {
                let descriptor = QwordAddressSpaceDescriptor {
                    resource_range_type: ResourceRangeType::from(bfr[0x03]),
                    general_flags: bfr[0x04],
                    type_specific_flags: bfr[0x05],
                    granularity: u64::from_le_bytes(bfr[0x06..0x06 + 8].try_into().unwrap()),
                    address_min: u64::from_le_bytes(bfr[0x0E..0x0E + 8].try_into().unwrap()),
                    address_max: u64::from_le_bytes(bfr[0x16..0x16 + 8].try_into().unwrap()),
                    translation_offset: u64::from_le_bytes(bfr[0x1E..0x1E + 8].try_into().unwrap()),
                    address_length: u64::from_le_bytes(bfr[0x26..0x26 + 8].try_into().unwrap()),
                };
                descriptors.push(descriptor);

                bfr = &bfr[3 + 0x2B..];
            }
            _ => break,
        }
    }

    descriptors
}

#[cfg(test)]
mod tests {
    use crate::proto::pci::configuration::ResourceRangeType;

    #[test]
    fn parse() {
        // example acpi pci qword configuration table export from a qemu vm
        const BFR: &[u8] = &[
            138, 43, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 96, 0, 0, 0, 0, 0, 0, 255, 111, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 138, 43, 0, 0, 0, 0, 32,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 255, 255, 15, 129, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 16, 1, 0, 0, 0, 0, 138, 43, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 192, 0, 0, 0, 255, 255, 15, 0, 192, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            16, 0, 0, 0, 0, 0, 138, 43, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 121, 0,
        ];
        let configuration = super::parse(BFR.as_ptr().cast());
        assert_eq!(configuration.len(), 4);
        let (mut cnt_mem, mut cnt_io, mut cnt_bus) = (0, 0, 0);
        for entry in &configuration {
            match entry.resource_range_type {
                ResourceRangeType::Memory => cnt_mem += 1,
                ResourceRangeType::Io => cnt_io += 1,
                ResourceRangeType::Bus => cnt_bus += 1,
                _ => unreachable!(),
            }
        }
        assert_eq!(cnt_mem, 2);
        assert_eq!(cnt_io, 1);
        assert_eq!(cnt_bus, 1);
    }
}
