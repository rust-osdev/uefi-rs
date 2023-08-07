use crate::{guid, Guid, Status};
use core::ffi::c_void;

/// Logical block address.
pub type Lba = u64;

/// Media information structure
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BlockIoMedia {
    pub media_id: u32,
    pub removable_media: bool,
    pub media_present: bool,
    pub logical_partition: bool,
    pub read_only: bool,
    pub write_caching: bool,
    pub block_size: u32,
    pub io_align: u32,
    pub last_block: Lba,

    // Added in revision 2.
    pub lowest_aligned_lba: Lba,
    pub logical_blocks_per_physical_block: u32,

    // Added in revision 3.
    pub optimal_transfer_length_granularity: u32,
}

#[repr(C)]
pub struct BlockIoProtocol {
    pub revision: u64,
    pub media: *const BlockIoMedia,
    pub reset: unsafe extern "efiapi" fn(this: *mut Self, extended_verification: bool) -> Status,
    pub read_blocks: unsafe extern "efiapi" fn(
        this: *const Self,
        media_id: u32,
        lba: Lba,
        buffer_size: usize,
        buffer: *mut c_void,
    ) -> Status,
    pub write_blocks: unsafe extern "efiapi" fn(
        this: *mut Self,
        media_id: u32,
        lba: Lba,
        buffer_size: usize,
        buffer: *const c_void,
    ) -> Status,
    pub flush_blocks: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
}

impl BlockIoProtocol {
    pub const GUID: Guid = guid!("964e5b21-6459-11d2-8e39-00a0c969723b");
}
