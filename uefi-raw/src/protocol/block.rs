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
