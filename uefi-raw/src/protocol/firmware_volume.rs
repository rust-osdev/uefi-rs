// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::firmware_storage::FirmwareVolumeAttributes;
use crate::protocol::block::Lba;
use crate::{guid, Guid, Handle, PhysicalAddress, Status};
use core::ffi::c_void;
use core::ops::RangeInclusive;

bitflags::bitflags! {
    /// EFI_FV_ATTRIBUTES
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[repr(transparent)]
    pub struct FvAttributes: u64 {
        const READ_DISABLE_CAP = 1 << 0;
        const READ_ENABLE_CAP = 1 << 1;
        const READ_STATUS = 1 << 2;

        const WRITE_DISABLE_CAP = 1 << 3;
        const WRITE_ENABLE_CAP = 1 << 4;
        const WRITE_STATUS = 1 << 5;

        const LOCK_CAP = 1 << 6;
        const LOCK_STATUS = 1 << 7;
        const WRITE_POLICY_RELIABLE = 1 << 8;
        const READ_LOCK_CAP = 1 << 12;
        const READ_LOCK_STATUS = 1 << 13;
        const WRITE_LOCK_CAP = 1 << 14;
        const WRITE_LOCK_STATUS = 1 << 15;

        const ALIGNMENT = 0x1F << 16;
        const ALIGNMENT_1 = 0x00 << 16;
        const ALIGNMENT_2 = 0x01 << 16;
        const ALIGNMENT_4 = 0x02 << 16;
        const ALIGNMENT_8 = 0x03 << 16;
        const ALIGNMENT_16 = 0x04 << 16;
        const ALIGNMENT_32 = 0x05 << 16;
        const ALIGNMENT_64 = 0x06 << 16;
        const ALIGNMENT_128 = 0x07 << 16;
        const ALIGNMENT_256 = 0x08 << 16;
        const ALIGNMENT_512 = 0x09 << 16;
        const ALIGNMENT_1K = 0x0A << 16;
        const ALIGNMENT_2K = 0x0B << 16;
        const ALIGNMENT_4K = 0x0C << 16;
        const ALIGNMENT_8K = 0x0D << 16;
        const ALIGNMENT_16K = 0x0E << 16;
        const ALIGNMENT_32K = 0x0F << 16;
        const ALIGNMENT_64K = 0x10 << 16;
        const ALIGNMENT_128K = 0x11 << 16;
        const ALIGNMENT_256K = 0x12 << 16;
        const ALIGNMENT_512K = 0x13 << 16;
        const ALIGNMENT_1M = 0x14 << 16;
        const ALIGNMENT_2M = 0x15 << 16;
        const ALIGNMENT_4M = 0x16 << 16;
        const ALIGNMENT_8M = 0x17 << 16;
        const ALIGNMENT_16M = 0x18 << 16;
        const ALIGNMENT_32M = 0x19 << 16;
        const ALIGNMENT_64M = 0x1A << 16;
        const ALIGNMENT_128M = 0x1B << 16;
        const ALIGNMENT_256M = 0x1C << 16;
        const ALIGNMENT_512M = 0x1D << 16;
        const ALIGNMENT_1G = 0x1E << 16;
        const ALIGNMENT_2G = 0x1F << 16;
    }
}

bitflags::bitflags! {
    /// EFI_FV_FILE_ATTRIBUTES
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[repr(transparent)]
    pub struct FvFileAttributes: u32 {
        const ALIGNMENT = 0x1F;
        const FIXED = 1 << 8;
        const MEMORY_MAPPED = 1 << 9;
    }
}

newtype_enum! {
    /// EFI_FV_WRITE_POLICY
    pub enum FvWritePolicy: u32 => {
        EFI_FV_UNRELIABLE_WRITE = 0,
        EFI_FV_RELIABLE_WRITE = 1,
    }
}

newtype_enum! {
    /// EFI_FV_FILETYPE
    pub enum FvFiletype: u8 => {
        ALL = 0x00,
        RAW = 0x01,
        FREEFORM = 0x02,
        SECURITY_CORE = 0x03,
        PEI_CORE = 0x04,
        DXE_CORE = 0x05,
        PEIM = 0x06,
        DRIVER = 0x07,
        COMBINED_PEIM_DRIVER = 0x08,
        APPLICATION = 0x09,
        MM = 0x0A,
        FIRMWARE_VOLUME_IMAGE = 0x0B,
        COMBINED_MM_DXE = 0x0C,
        MM_CORE = 0x0D,
        MM_STANDALONE = 0x0E,
        MM_CORE_STANDALONE = 0x0F,
        FFS_PAD = 0xF0,
    }
}

impl FvFiletype {
    pub const OEM_RANGE: RangeInclusive<u8> = 0xC0..=0xDF;
    pub const DEBUG_RANGE: RangeInclusive<u8> = 0xE0..=0xEF;
    pub const FFS_RANGE: RangeInclusive<u8> = 0xF0..=0xFF;
}

newtype_enum! {
    /// EFI_SECTION_TYPE
    pub enum SectionType: u8 => {
        ALL = 0x00,
        COMPRESSION = 0x01,
        GUID_DEFINED = 0x02,
        DISPOSABLE = 0x03,
        PE32 = 0x10,
        PIC = 0x11,
        TE = 0x12,
        DXE_DEPEX = 0x13,
        VERSION = 0x14,
        USER_INTERFACE = 0x15,
        COMPATIBILITY16 = 0x16,
        FIRMWARE_VOLUME_IMAGE = 0x17,
        FREEFORM_SUBTYPE_GUID = 0x18,
        RAW = 0x19,
        PEI_DEPEX = 0x1B,
        MM_DEPEX = 0x1C,
    }
}

/// EFI_FV_WRITE_FILE_DATA
#[derive(Debug)]
#[repr(C)]
pub struct FvWriteFileData {
    pub name_guid: *const Guid,
    pub r#type: FvFiletype,
    pub file_attributes: FvFileAttributes,
    pub buffer: *const u8,
    pub buffer_size: u32,
}

/// EFI_FIRMWARE_VOLUME2_PROTOCOL
#[derive(Debug)]
#[repr(C)]
pub struct FirmwareVolume2Protocol {
    pub get_volume_attributes:
        unsafe extern "efiapi" fn(this: *const Self, fv_attributes: *mut FvAttributes) -> Status,
    pub set_volume_attributes:
        unsafe extern "efiapi" fn(this: *const Self, fv_attributes: *mut FvAttributes) -> Status,
    pub read_file: unsafe extern "efiapi" fn(
        this: *const Self,
        name_guid: *const Guid,
        buffer: *mut *mut c_void,
        buffer_size: *mut usize,
        found_type: *mut FvFiletype,
        file_attributes: *mut FvFileAttributes,
        authentication_status: *mut u32,
    ) -> Status,
    pub read_section: unsafe extern "efiapi" fn(
        this: *const Self,
        name_guid: *const Guid,
        section_type: SectionType,
        section_instance: usize,
        buffer: *mut *mut c_void,
        buffer_size: *mut usize,
        authentication_status: *mut u32,
    ) -> Status,
    pub write_file: unsafe extern "efiapi" fn(
        this: *const Self,
        number_of_files: u32,
        write_policy: FvWritePolicy,
        file_data: *const FvWriteFileData,
    ) -> Status,
    pub get_next_file: unsafe extern "efiapi" fn(
        this: *const Self,
        key: *mut c_void,
        file_type: *mut FvFiletype,
        name_guid: *mut Guid,
        attributes: *mut FvFileAttributes,
        size: *mut usize,
    ) -> Status,
    pub key_size: u32,
    pub parent_handle: Handle,
    pub get_info: unsafe extern "efiapi" fn(
        this: *const Self,
        information_type: *const Guid,
        buffer_size: *mut usize,
        buffer: *mut c_void,
    ) -> Status,
    pub set_info: unsafe extern "efiapi" fn(
        this: *const Self,
        information_type: *const Guid,
        buffer_size: usize,
        buffer: *const c_void,
    ) -> Status,
}

impl FirmwareVolume2Protocol {
    pub const GUID: Guid = guid!("220e73b6-6bdb-4413-8405-b974b108619a");
}

/// EFI_FIRMWARE_VOLUME_BLOCK2_PROTOCOL
#[derive(Debug)]
#[repr(C)]
pub struct FirmwareVolumeBlock2Protocol {
    pub get_attributes: unsafe extern "efiapi" fn(
        this: *const Self,
        attributes: *mut FirmwareVolumeAttributes,
    ) -> Status,
    pub set_attributes: unsafe extern "efiapi" fn(
        this: *const Self,
        attributes: *mut FirmwareVolumeAttributes,
    ) -> Status,
    pub get_physical_address:
        unsafe extern "efiapi" fn(this: *const Self, address: *mut PhysicalAddress) -> Status,
    pub get_block_size: unsafe extern "efiapi" fn(
        this: *const Self,
        lba: Lba,
        block_size: *mut usize,
        number_of_blocks: *mut usize,
    ) -> Status,
    pub read: unsafe extern "efiapi" fn(
        this: *const Self,
        lba: Lba,
        offset: usize,
        num_bytes: *mut usize,
        buffer: *mut u8,
    ) -> Status,
    pub write: unsafe extern "efiapi" fn(
        this: *const Self,
        lba: Lba,
        offset: usize,
        num_bytes: *mut usize,
        buffer: *mut u8,
    ) -> Status,
    // TODO: Change to efiapi (https://github.com/rust-lang/rust/issues/100189)
    pub erase_blocks: unsafe extern "C" fn(this: *const Self, ...) -> Status,
    pub parent_handle: Handle,
}

impl FirmwareVolumeBlock2Protocol {
    pub const GUID: Guid = guid!("8f644fa9-e850-4db1-9ce2-0b44698e8da4");
    pub const LBA_LIST_TERMINATOR: u64 = u64::MAX;
}
