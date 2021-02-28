//! Block I/O protocols.

use crate::proto::Protocol;
use crate::{unsafe_guid, Result, Status};

/// The Block I/O protocol.
#[repr(C)]
#[unsafe_guid("964e5b21-6459-11d2-8e39-00a0c969723b")]
#[derive(Protocol)]
pub struct BlockIO {
    revision: u64,
    media: BlockIOMedia,

    reset: extern "efiapi" fn(this: &BlockIO, extended_verification: bool) -> Status,
    read_blocks: extern "efiapi" fn(
        this: &BlockIO,
        media_id: u32,
        lba: Lba,
        buffer_size: usize,
        buffer: *mut u8,
    ) -> Status,
    write_blocks: extern "efiapi" fn(
        this: &BlockIO,
        media_id: u32,
        lba: Lba,
        buffer_size: usize,
        buffer: *const u8,
    ) -> Status,
    flush_blocks: extern "efiapi" fn(this: &BlockIO) -> Status,
}

impl BlockIO {
    /// Pointer for block IO media.
    pub fn media(&self) -> &BlockIOMedia {
        &self.media
    }

    /// Resets the block device hardware.
    ///
    /// # Arguments
    /// * `extended_verification`   Indicates that the driver may perform a more exhaustive verification operation of
    ///     the device during reset.
    ///
    /// # Errors
    /// * `uefi::Status::DEVICE_ERROR`  The block device is not functioning correctly and could not be reset.
    pub fn reset(&mut self, extended_verification: bool) -> Result {
        (self.reset)(self, extended_verification).into()
    }

    /// Read the requested number of blocks from the device.
    ///
    /// # Arguments
    /// * `media_id` - The media ID that the read request is for.
    /// * `lba` - The starting logical block address to read from on the device.
    /// * `buffer` - The target buffer of the read operation
    ///
    /// # Errors
    /// * `uefi::Status::DEVICE_ERROR`       The device reported an error while attempting to perform the read
    ///     operation.
    /// * `uefi::Status::NO_MEDIA`           There is no media in the device.
    /// * `uefi::Status::MEDIA_CHANGED`      The `media_id` is not for the current media.
    /// * `uefi::Status::BAD_BUFFER_SIZE`    The buffer size parameter is not a multiple of the intrinsic block size of
    ///     the device.
    /// * `uefi::Status::INVALID_PARAMETER`  The read request contains LBAs that are not valid, or the buffer is not on
    ///     proper alignment.
    pub fn read_blocks(&self, media_id: u32, lba: Lba, buffer: &mut [u8]) -> Result {
        let buffer_size = buffer.len();
        (self.read_blocks)(self, media_id, lba, buffer_size, buffer.as_mut_ptr()).into()
    }

    /// Writes the requested number of blocks to the device.
    ///
    /// # Arguments
    /// * `media_id`    The media ID that the write request is for.
    /// * `lba`         The starting logical block address to be written.
    /// * `buffer`      Buffer to be written
    ///
    /// # Errors
    /// * `uefi::Status::WRITE_PROTECTED`       The device cannot be written to.
    /// * `uefi::Status::NO_MEDIA`              There is no media in the device.
    /// * `uefi::Status::MEDIA_CHANGED`         The `media_id` is not for the current media.
    /// * `uefi::Status::DEVICE_ERROR`          The device reported an error while attempting to perform the write
    ///     operation.
    /// * `uefi::Status::BAD_BUFFER_SIZE`       The buffer size parameter is not a multiple of the intrinsic block size
    ///     of the device.
    /// * `uefi::Status::INVALID_PARAMETER`     The write request contains LBAs that are not valid, or the buffer is not
    ///     on proper alignment.
    pub fn write_blocks(&mut self, media_id: u32, lba: Lba, buffer: &[u8]) -> Result {
        let buffer_size = buffer.len();
        (self.write_blocks)(self, media_id, lba, buffer_size, buffer.as_ptr()).into()
    }

    /// Flushes all modified data to a physical block device.
    ///
    /// # Errors
    /// * `uefi::Status::DEVICE_ERROR`          The device reported an error while attempting to write data.
    /// * `uefi::Status::NO_MEDIA`              There is no media in the device.
    pub fn flush_blocks(&mut self) -> Result {
        (self.flush_blocks)(self).into()
    }
}

/// EFI LBA type
pub type Lba = u64;

/// Media information structure
#[repr(C)]
#[derive(Debug)]
pub struct BlockIOMedia {
    media_id: u32,
    removable_media: bool,
    media_present: bool,
    logical_partition: bool,
    read_only: bool,
    write_caching: bool,

    block_size: u32,
    io_align: u32,
    last_block: Lba,

    // Revision 2
    lowest_aligned_lba: Lba,
    logical_blocks_per_physical_block: u32,

    // Revision 3
    optimal_transfer_length_granularity: u32,
}

impl BlockIOMedia {
    /// The current media ID.
    pub fn media_id(&self) -> u32 {
        self.media_id
    }

    /// True if the media is removable.
    pub fn is_removable_media(&self) -> bool {
        self.removable_media
    }

    /// True if there is a media currently present in the device.
    pub fn is_media_preset(&self) -> bool {
        self.media_present
    }

    /// True if block IO was produced to abstract partition structure.
    pub fn is_logical_partition(&self) -> bool {
        self.logical_partition
    }

    /// True if the media is marked read-only.
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// True if `writeBlocks` function writes data.
    pub fn is_write_caching(&self) -> bool {
        self.write_caching
    }

    /// The intrinsic block size of the device.
    ///
    /// If the media changes, then this field is updated. Returns the number of bytes per logical block.
    pub fn block_size(&self) -> u32 {
        self.block_size
    }

    /// Supplies the alignment requirement for any buffer used in a data transfer.
    pub fn io_align(&self) -> u32 {
        self.io_align
    }

    /// The last LBA on the device. If the media changes, then this field is updated.
    pub fn last_block(&self) -> Lba {
        self.last_block
    }

    /// Returns the first LBA that is aligned to a physical block boundary.
    pub fn lowest_aligned_lba(&self) -> Lba {
        self.lowest_aligned_lba
    }

    /// Returns the number of logical blocks per physical block.
    pub fn logical_blocks_per_physical_block(&self) -> u32 {
        self.logical_blocks_per_physical_block
    }

    /// Returns the optimal transfer length granularity as a number of logical blocks.
    pub fn optimal_transfer_length_granularity(&self) -> u32 {
        self.optimal_transfer_length_granularity
    }
}
