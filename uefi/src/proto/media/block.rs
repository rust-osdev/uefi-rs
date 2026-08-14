// SPDX-License-Identifier: MIT OR Apache-2.0

//! Block I/O protocols [`BlockIO`] and [`BlockIO2`].

use core::ptr::NonNull;

use crate::proto::unsafe_protocol;
use crate::util::opt_nonnull_to_ptr;
use crate::{Event, Result, Status, StatusExt};

pub use uefi_raw::protocol::block::{BlockIo2Protocol, BlockIoProtocol, Lba};

/// Block I/O [`Protocol`].
///
/// [`Protocol`]: uefi::proto::Protocol
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(BlockIoProtocol::GUID)]
pub struct BlockIO(BlockIoProtocol);

impl BlockIO {
    /// Returns block I/O media information.
    #[must_use]
    pub const fn media(&self) -> &BlockIOMedia {
        // SAFETY: The memory is valid.
        unsafe { &*self.0.media.cast::<BlockIOMedia>() }
    }

    /// Resets the block device hardware.
    ///
    /// # Arguments
    ///
    /// - `extended_verification`: Requests an exhaustive device verification
    ///   during reset.
    ///
    /// # Errors
    ///
    /// - [`Status::DEVICE_ERROR`]: the device is malfunctioning and could not
    ///   be reset.
    pub fn reset(&mut self, extended_verification: bool) -> Result {
        // SAFETY: The memory is valid.
        unsafe { (self.0.reset)(&mut self.0, extended_verification.into()) }.to_result()
    }

    /// Reads blocks from the device.
    ///
    /// # Arguments
    ///
    /// - `media_id`: Identifier of the medium to read.
    /// - `lba`: First logical block to read.
    /// - `buffer`: Destination buffer; its length determines the block count.
    ///
    /// # Errors
    ///
    /// - [`Status::DEVICE_ERROR`]: the device reported an error.
    /// - [`Status::NO_MEDIA`]: no medium is present.
    /// - [`Status::MEDIA_CHANGED`]: `media_id` is not current.
    /// - [`Status::BAD_BUFFER_SIZE`]: the buffer length is not a multiple of
    ///   the block size.
    /// - [`Status::INVALID_PARAMETER`]: an LBA is invalid or the buffer is
    ///   incorrectly aligned.
    pub fn read_blocks(&self, media_id: u32, lba: Lba, buffer: &mut [u8]) -> Result {
        let buffer_size = buffer.len();
        // SAFETY: The memory is valid.
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
    ///
    /// - `media_id`: Identifier of the medium to write.
    /// - `lba`: First logical block to write.
    /// - `buffer`: Source buffer; its length determines the block count.
    ///
    /// # Errors
    ///
    /// - [`Status::WRITE_PROTECTED`]: the device is read-only.
    /// - [`Status::NO_MEDIA`]: no medium is present.
    /// - [`Status::MEDIA_CHANGED`]: `media_id` is not current.
    /// - [`Status::DEVICE_ERROR`]: the device reported an error.
    /// - [`Status::BAD_BUFFER_SIZE`]: the buffer length is not a multiple of
    ///   the block size.
    /// - [`Status::INVALID_PARAMETER`]: an LBA is invalid or the buffer is
    ///   incorrectly aligned.
    pub fn write_blocks(&mut self, media_id: u32, lba: Lba, buffer: &[u8]) -> Result {
        let buffer_size = buffer.len();
        // SAFETY: The memory is valid.
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
    ///
    /// - [`Status::DEVICE_ERROR`]: the device reported an error.
    /// - [`Status::NO_MEDIA`]: no medium is present.
    pub fn flush_blocks(&mut self) -> Result {
        // SAFETY: The memory is valid.
        unsafe { (self.0.flush_blocks)(&mut self.0) }.to_result()
    }
}

/// Describes the medium exposed by a block I/O device.
#[repr(transparent)]
#[derive(Debug)]
pub struct BlockIOMedia(uefi_raw::protocol::block::BlockIoMedia);

impl BlockIOMedia {
    /// Returns the current media ID.
    #[must_use]
    pub const fn media_id(&self) -> u32 {
        self.0.media_id
    }

    /// Returns whether the media is removable.
    #[must_use]
    pub fn is_removable_media(&self) -> bool {
        self.0.removable_media.into()
    }

    /// Returns whether media is currently present in the device.
    #[must_use]
    pub fn is_media_present(&self) -> bool {
        self.0.media_present.into()
    }

    /// Returns whether block I/O abstracts a partition.
    #[must_use]
    pub fn is_logical_partition(&self) -> bool {
        self.0.logical_partition.into()
    }

    /// Returns whether the media is read-only.
    #[must_use]
    pub fn is_read_only(&self) -> bool {
        self.0.read_only.into()
    }

    /// Returns whether the `write_blocks` function writes data.
    #[must_use]
    pub fn is_write_caching(&self) -> bool {
        self.0.write_caching.into()
    }

    /// Returns the intrinsic block size in bytes.
    ///
    /// This value is updated when the medium changes.
    #[must_use]
    pub const fn block_size(&self) -> u32 {
        self.0.block_size
    }

    /// Returns the alignment required for data-transfer buffers.
    #[must_use]
    pub const fn io_align(&self) -> u32 {
        self.0.io_align
    }

    /// Returns the last LBA on the device.
    ///
    /// This value is updated when the medium changes.
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

/// Asynchronous transaction token for Block I/O 2 operations.
#[repr(C)]
#[derive(Debug)]
pub struct BlockIO2Token {
    /// Event to be signalled when an asynchronous block I/O operation
    /// completes.
    pub event: Option<Event>,
    /// Transaction status code.
    pub transaction_status: Status,
}

/// Block I/O 2 [`Protocol`].
///
/// The Block I/O 2 protocol defines an extension to the Block I/O protocol
/// which enables the ability to read and write data at a block level in a
/// non-blocking manner.
///
/// [`Protocol`]: uefi::proto::Protocol
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(BlockIo2Protocol::GUID)]
pub struct BlockIO2(BlockIo2Protocol);

impl BlockIO2 {
    /// Returns block I/O media information.
    #[must_use]
    pub const fn media(&self) -> &BlockIOMedia {
        // SAFETY: The memory is valid.
        unsafe { &*self.0.media.cast::<BlockIOMedia>() }
    }

    /// Resets the block device hardware.
    ///
    /// # Arguments
    ///
    /// - `extended_verification`: Requests an exhaustive device verification
    ///   during reset.
    ///
    /// # Errors
    ///
    /// - [`Status::DEVICE_ERROR`]: the device is malfunctioning and could not
    ///   be reset.
    pub fn reset(&mut self, extended_verification: bool) -> Result {
        // SAFETY: The memory is valid.
        unsafe { (self.0.reset)(&mut self.0, extended_verification.into()) }.to_result()
    }

    /// Reads the requested number of blocks from the device.
    ///
    /// # Arguments
    ///
    /// - `media_id`: Identifier of the medium to read.
    /// - `lba`: First logical block to read.
    /// - `token`: Optional transaction token for asynchronous completion.
    /// - `len`: Number of bytes available at `buffer`.
    /// - `buffer`: Destination buffer.
    ///
    /// # Safety
    /// `token` and `buffer` must remain valid until the transaction completes.
    ///
    /// # Errors
    ///
    /// - [`Status::INVALID_PARAMETER`]: an LBA is invalid or the buffer is
    ///   incorrectly aligned.
    /// - [`Status::OUT_OF_RESOURCES`]: insufficient resources for the request.
    /// - [`Status::MEDIA_CHANGED`]: `media_id` is not current.
    /// - [`Status::NO_MEDIA`]: no medium is present.
    /// - [`Status::DEVICE_ERROR`]: the device reported an error.
    /// - [`Status::BAD_BUFFER_SIZE`]: `len` is not a multiple of the block
    ///   size.
    pub unsafe fn read_blocks_ex(
        &self,
        media_id: u32,
        lba: Lba,
        token: Option<NonNull<BlockIO2Token>>,
        len: usize,
        buffer: *mut u8,
    ) -> Result {
        let token = opt_nonnull_to_ptr(token);
        // SAFETY: The memory is valid.
        unsafe { (self.0.read_blocks_ex)(&self.0, media_id, lba, token.cast(), len, buffer.cast()) }
            .to_result()
    }

    /// Writes a specified number of blocks to the device.
    ///
    /// # Arguments
    ///
    /// - `media_id`: Identifier of the medium to write.
    /// - `lba`: First logical block to write.
    /// - `token`: Optional transaction token for asynchronous completion.
    /// - `len`: Number of bytes available at `buffer`.
    /// - `buffer`: Source buffer.
    ///
    /// # Safety
    /// `token` and `buffer` must remain valid until the transaction completes.
    ///
    /// # Errors
    ///
    /// - [`Status::INVALID_PARAMETER`]: an LBA is invalid or the buffer is
    ///   incorrectly aligned.
    /// - [`Status::OUT_OF_RESOURCES`]: insufficient resources for the request.
    /// - [`Status::MEDIA_CHANGED`]: `media_id` is not current.
    /// - [`Status::NO_MEDIA`]: no medium is present.
    /// - [`Status::DEVICE_ERROR`]: the device reported an error.
    /// - [`Status::WRITE_PROTECTED`]: the device is read-only.
    /// - [`Status::BAD_BUFFER_SIZE`]: `len` is not a multiple of the block
    ///   size.
    pub unsafe fn write_blocks_ex(
        &mut self,
        media_id: u32,
        lba: Lba,
        token: Option<NonNull<BlockIO2Token>>,
        len: usize,
        buffer: *const u8,
    ) -> Result {
        let token = opt_nonnull_to_ptr(token);
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.write_blocks_ex)(&mut self.0, media_id, lba, token.cast(), len, buffer.cast())
        }
        .to_result()
    }

    /// Flushes all modified data to the physical device.
    ///
    /// # Arguments
    ///
    /// - `token`: Optional transaction token for asynchronous completion.
    ///
    /// # Errors
    ///
    /// - [`Status::OUT_OF_RESOURCES`]: insufficient resources for the request.
    /// - [`Status::MEDIA_CHANGED`]: the medium changed since the last access.
    /// - [`Status::NO_MEDIA`]: no medium is present.
    /// - [`Status::DEVICE_ERROR`]: the device reported an error.
    /// - [`Status::WRITE_PROTECTED`]: the device is read-only.
    pub fn flush_blocks_ex(&mut self, token: Option<NonNull<BlockIO2Token>>) -> Result {
        let token = opt_nonnull_to_ptr(token);
        // SAFETY: The memory is valid.
        unsafe { (self.0.flush_blocks_ex)(&mut self.0, token.cast()) }.to_result()
    }
}
