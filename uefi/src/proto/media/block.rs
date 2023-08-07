//! Block I/O protocols.

use crate::proto::unsafe_protocol;
use crate::{Result, StatusExt};

pub use uefi_raw::protocol::block::{BlockIoProtocol, Lba};

/// The Block I/O protocol.
#[repr(transparent)]
#[unsafe_protocol(BlockIoProtocol::GUID)]
pub struct BlockIO(BlockIoProtocol);

impl BlockIO {
    /// Pointer for block IO media.
    #[must_use]
    pub const fn media(&self) -> &BlockIOMedia {
        unsafe { &*self.0.media.cast::<BlockIOMedia>() }
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
        unsafe { (self.0.reset)(&mut self.0, extended_verification) }.to_result()
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
        unsafe {
            (self.0.read_blocks)(
                &self.0,
                media_id,
                lba,
                buffer_size,
                buffer.as_mut_ptr().cast(),
            )
        }
        .to_result()
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
        unsafe {
            (self.0.write_blocks)(
                &mut self.0,
                media_id,
                lba,
                buffer_size,
                buffer.as_ptr().cast(),
            )
        }
        .to_result()
    }

    /// Flushes all modified data to a physical block device.
    ///
    /// # Errors
    /// * `uefi::Status::DEVICE_ERROR`          The device reported an error while attempting to write data.
    /// * `uefi::Status::NO_MEDIA`              There is no media in the device.
    pub fn flush_blocks(&mut self) -> Result {
        unsafe { (self.0.flush_blocks)(&mut self.0) }.to_result()
    }
}

/// Media information structure
#[repr(transparent)]
#[derive(Debug)]
pub struct BlockIOMedia(uefi_raw::protocol::block::BlockIoMedia);

impl BlockIOMedia {
    /// The current media ID.
    #[must_use]
    pub const fn media_id(&self) -> u32 {
        self.0.media_id
    }

    /// True if the media is removable.
    #[must_use]
    pub const fn is_removable_media(&self) -> bool {
        self.0.removable_media
    }

    /// True if there is a media currently present in the device.
    #[must_use]
    pub const fn is_media_present(&self) -> bool {
        self.0.media_present
    }

    /// True if block IO was produced to abstract partition structure.
    #[must_use]
    pub const fn is_logical_partition(&self) -> bool {
        self.0.logical_partition
    }

    /// True if the media is marked read-only.
    #[must_use]
    pub const fn is_read_only(&self) -> bool {
        self.0.read_only
    }

    /// True if `writeBlocks` function writes data.
    #[must_use]
    pub const fn is_write_caching(&self) -> bool {
        self.0.write_caching
    }

    /// The intrinsic block size of the device.
    ///
    /// If the media changes, then this field is updated. Returns the number of bytes per logical block.
    #[must_use]
    pub const fn block_size(&self) -> u32 {
        self.0.block_size
    }

    /// Supplies the alignment requirement for any buffer used in a data transfer.
    #[must_use]
    pub const fn io_align(&self) -> u32 {
        self.0.io_align
    }

    /// The last LBA on the device. If the media changes, then this field is updated.
    #[must_use]
    pub const fn last_block(&self) -> Lba {
        self.0.last_block
    }

    /// Returns the first LBA that is aligned to a physical block boundary.
    #[must_use]
    pub const fn lowest_aligned_lba(&self) -> Lba {
        self.0.lowest_aligned_lba
    }

    /// Returns the number of logical blocks per physical block.
    #[must_use]
    pub const fn logical_blocks_per_physical_block(&self) -> u32 {
        self.0.logical_blocks_per_physical_block
    }

    /// Returns the optimal transfer length granularity as a number of logical blocks.
    #[must_use]
    pub const fn optimal_transfer_length_granularity(&self) -> u32 {
        self.0.optimal_transfer_length_granularity
    }
}
