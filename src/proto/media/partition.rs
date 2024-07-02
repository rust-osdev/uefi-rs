//! Partition information protocol.

use crate::proto::Protocol;
use crate::{unsafe_guid, Char16, Guid};
use bitflags::bitflags;

newtype_enum! {
    /// MBR OS type.
    ///
    /// Only two values are defined in the UEFI specification, other
    /// values are used by legacy operating systems.
    pub enum MbrOsType: u8 => {
        /// A fake partition covering the entire disk.
        GPT_PROTECTIVE = 0xee,

        /// UEFI system partition.
        UEFI_SYSTEM_PARTITION = 0xef,
    }
}

/// Legacy MBR Partition Record.
#[repr(C)]
#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct MbrPartitionRecord {
    /// If 0x80, this is the bootable legacy partition.
    pub boot_indicator: u8,

    /// Start of the partition in CHS address format.
    pub starting_chs: [u8; 3],

    /// Type of partition.
    pub os_type: MbrOsType,

    /// End of the partition in CHS address format.
    pub ending_chs: [u8; 3],

    /// Starting LBA of the partition on the disk.
    pub starting_lba: u32,

    /// Size of the partition in LBA units of logical blocks.
    pub size_in_lba: u32,
}

impl MbrPartitionRecord {
    /// True if the partition is a bootable legacy partition.
    pub fn is_bootable(&self) -> bool {
        self.boot_indicator == 0x80
    }
}

newtype_enum! {
    /// GUID that defines the type of partition. Only three values are
    /// defined in the UEFI specification, OS vendors define their own
    /// Partition Type GUIDs.
    pub enum GptPartitionType: Guid => {
        /// Indicates a partition entry is unused.
        UNUSED_ENTRY = Guid::from_values(
            0x00000000,
            0x0000,
            0x0000,
            0x0000,
            0x000000000000,
        ),

        /// EFI System Partition.
        EFI_SYSTEM_PARTITION = Guid::from_values(
            0xc12a7328,
            0xf81f,
            0x11d2,
            0xba4b,
            0x00a0c93ec93b,
        ),

        /// Partition containing a legacy MBR.
        LEGACY_MBR = Guid::from_values(
            0x024dee41,
            0x33e7,
            0x11d3,
            0x9d69,
            0x0008c781f39f,
        ),
    }
}

bitflags! {
    /// Attributes describing a GPT partition.
    ///
    /// * Bit 0: [`REQUIRED_PARTITION`][Self::REQUIRED_PARTITION]
    /// * Bit 1: [`NO_BLOCK_IO_PROTOCOL`][Self::NO_BLOCK_IO_PROTOCOL]
    /// * Bit 2: [`LEGACY_BIOS_BOOTABLE`][Self::LEGACY_BIOS_BOOTABLE]
    /// * Bits `3..=47`: reserved for future use and must be zero.
    /// * Bits `48..=63`: See
    /// [`type_specific_bits`][Self::type_specific_bits] and
    /// [`RESERVED_FOR_PARTITION_TYPE`][Self::RESERVED_FOR_PARTITION_TYPE].
    #[derive(Default)]
    #[repr(transparent)]
    pub struct GptPartitionAttributes: u64 {
        /// Partition is required for the platform to function.
        const REQUIRED_PARTITION = 1 << 0;

        /// No [`BlockIO`] protocol will be created for this partition.
        ///
        /// [`BlockIO`]: uefi::proto::media::block::BlockIO
        const NO_BLOCK_IO_PROTOCOL = 1 << 1;

        /// Indicates that special software on a legacy BIOS system may
        /// treat this partition as bootable. UEFI boot managers must
        /// ignore the partition.
        const LEGACY_BIOS_BOOTABLE = 1 << 2;

        /// Mask for bits `48..=63`. The meaning of these bits depends
        /// on the partition type.
        const RESERVED_FOR_PARTITION_TYPE = 0xffff_0000_0000_0000;
    }
}

impl GptPartitionAttributes {
    /// Get bits `48..=63` as a [`u16`]. The meaning of these bits depends
    /// on the partition's type (see [`GptPartitionEntry::partition_type_guid`]).
    pub fn type_specific_bits(&self) -> u16 {
        (self.bits >> 48) as u16
    }
}

/// GPT/EFI Partition Entry.
#[repr(C)]
#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct GptPartitionEntry {
    /// GUID that defines the type of this Partition. A value of zero
    /// indicates that this partition entry is unused.
    pub partition_type_guid: GptPartitionType,

    /// GUID that is unique for every partition entry.
    pub unique_partition_guid: Guid,

    /// Starting LBA of the partition.
    pub starting_lba: u64,

    /// Ending LBA of the partition.
    pub ending_lba: u64,

    /// All attribute bits of the partition.
    pub attributes: GptPartitionAttributes,

    /// Null-terminated string containing a human-readable name of the
    /// partition.
    pub partition_name: [Char16; 36],
}

impl GptPartitionEntry {
    /// Get the number of blocks in the partition. Returns `None` if the
    /// end block is before the start block, or if the number doesn't
    /// fit in a `u64`.
    pub fn num_blocks(&self) -> Option<u64> {
        self.ending_lba
            .checked_sub(self.starting_lba)?
            .checked_add(1)
    }
}

newtype_enum! {
    /// Partition type.
    pub enum PartitionType: u32 => {
        /// Partition is not MBR or GPT.
        OTHER = 0x00,
        /// MBR partition.
        MBR = 0x01,
        /// GPT partition.
        GPT = 0x02,
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
union PartitionInfoRecord {
    mbr: MbrPartitionRecord,
    gpt: GptPartitionEntry,
}

newtype_enum! {
    /// Partition info protocol revision.
    pub enum PartitionInfoRevision: u32 => {
        /// Revision of EFI_PARTITION_INFO_PROTOCOL_REVISION.
        PROTOCOL_REVISION = 0x0001000,
    }
}

/// Protocol for accessing partition information.
#[repr(C)]
#[repr(packed)]
#[unsafe_guid("8cf2f62c-bc9b-4821-808d-ec9ec421a1a0")]
#[derive(Clone, Copy, Protocol)]
pub struct PartitionInfo {
    /// Revision of the partition info protocol.
    pub revision: PartitionInfoRevision,

    /// Type of partition.
    pub partition_type: PartitionType,

    system: u8,
    reserved: [u8; 7],
    record: PartitionInfoRecord,
}

impl PartitionInfo {
    /// True if the partition is an EFI system partition.
    pub fn is_system(&self) -> bool {
        self.system == 1
    }

    /// Get the MBR partition record. Returns None if the partition
    /// type is not MBR.
    pub fn mbr_partition_record(&self) -> Option<&MbrPartitionRecord> {
        if { self.revision } != PartitionInfoRevision::PROTOCOL_REVISION {
            return None;
        }

        if { self.partition_type } == PartitionType::MBR {
            Some(unsafe { &self.record.mbr })
        } else {
            None
        }
    }

    /// Get the GPT partition entry. Returns None if the partition
    /// type is not GPT.
    pub fn gpt_partition_entry(&self) -> Option<&GptPartitionEntry> {
        if { self.revision } != PartitionInfoRevision::PROTOCOL_REVISION {
            return None;
        }

        if { self.partition_type } == PartitionType::GPT {
            Some(unsafe { &self.record.gpt })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_attributes() {
        let attr = GptPartitionAttributes::from_bits(0xabcd_0000_0000_0007).unwrap();
        assert_eq!(attr.type_specific_bits(), 0xabcd);
    }
}
