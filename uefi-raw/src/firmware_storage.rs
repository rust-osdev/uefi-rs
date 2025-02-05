// SPDX-License-Identifier: MIT OR Apache-2.0

//! Types related to firmware storage.

use crate::Guid;
use bitflags::bitflags;

/// Corresponds to the C type `EFI_FIRMWARE_VOLUME_HEADER`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(C)]
pub struct FirmwareVolumeHeader {
    pub zero_vector: [u8; 16],
    pub file_system_guid: Guid,
    pub fv_length: u64,
    pub signature: [u8; 4],
    pub attributes: FirmwareVolumeAttributes,
    pub header_length: u16,
    pub checksum: u16,
    pub ext_header_offset: u16,
    pub reserved: u8,
    pub revision: u8,

    /// Variable-length array of block maps, terminated with a zero-filled
    /// entry.
    pub block_map: [FirmwareVolumeBlockMap; 0],
}

impl FirmwareVolumeHeader {
    pub const SIGNATURE: [u8; 4] = *b"_FVH";
}

/// Corresponds to the C type `EFI_FV_BLOCK_MAP`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(C)]
pub struct FirmwareVolumeBlockMap {
    pub num_blocks: u32,
    pub length: u32,
}

bitflags! {
    /// Corresponds to the C type `EFI_FVB_ATTRIBUTES_2`.
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[repr(transparent)]
    pub struct FirmwareVolumeAttributes: u32 {
        const READ_DISABLED_CAP = 0x0000_0001;
        const READ_ENABLED_CAP = 0x0000_0002;
        const READ_STATUS = 0x0000_0004;

        const WRITE_DISABLED_CAP = 0x0000_0008;
        const WRITE_ENABLED_CAP = 0x0000_0010;
        const WRITE_STATUS = 0x0000_0020;

        const LOCK_CAP = 0x0000_0040;
        const LOCK_STATUS = 0x0000_0080;

        const STICKY_WRITE = 0x0000_0200;
        const MEMORY_MAPPED = 0x0000_0400;
        const ERASE_POLARITY = 0x0000_0800;

        const READ_LOCK_CAP = 0x0000_1000;
        const READ_LOCK_STATUS = 0x0000_2000;

        const WRITE_LOCK_CAP = 0x0000_4000;
        const WRITE_LOCK_STATUS = 0x0000_8000;

        const ALIGNMENT = 0x001f_0000;
        const WEAK_ALIGNMENT = 0x8000_0000;
        const ALIGNMENT_1 = 0x0000_0000;
        const ALIGNMENT_2 = 0x0001_0000;
        const ALIGNMENT_4 = 0x0002_0000;
        const ALIGNMENT_8 = 0x0003_0000;
        const ALIGNMENT_16 = 0x0004_0000;
        const ALIGNMENT_32 = 0x0005_0000;
        const ALIGNMENT_64 = 0x0006_0000;
        const ALIGNMENT_128 = 0x0007_0000;
        const ALIGNMENT_256 = 0x0008_0000;
        const ALIGNMENT_512 = 0x0008_0000;
        const ALIGNMENT_1K = 0x000a_0000;
        const ALIGNMENT_2K = 0x000b_0000;
        const ALIGNMENT_4K = 0x000c_0000;
        const ALIGNMENT_8K = 0x000d_0000;
        const ALIGNMENT_16K = 0x000e_0000;
        const ALIGNMENT_32K = 0x000f_0000;
        const ALIGNMENT_64K = 0x0010_0000;
        const ALIGNMENT_128K = 0x0011_0000;
        const ALIGNMENT_256K = 0x0012_0000;
        const ALIGNMENT_512K = 0x0013_0000;
        const ALIGNMENT_1M = 0x0014_0000;
        const ALIGNMENT_2M = 0x0015_0000;
        const ALIGNMENT_4M = 0x0016_0000;
        const ALIGNMENT_8M = 0x0017_0000;
        const ALIGNMENT_16M = 0x0018_0000;
        const ALIGNMENT_32M = 0x0019_0000;
        const ALIGNMENT_64M = 0x001a_0000;
        const ALIGNMENT_128M = 0x001b_0000;
        const ALIGNMENT_256M = 0x001c_0000;
        const ALIGNMENT_512M = 0x001d_0000;
        const ALIGNMENT_1G = 0x001e_0000;
        const ALIGNMENT_2G = 0x001f_0000;
    }
}
