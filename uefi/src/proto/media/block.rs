// SPDX-License-Identifier: MIT OR Apache-2.0

//! Block I/O protocols [`BlockIO`] and [`BlockIO2`].

use crate::proto::unsafe_protocol;
use crate::util::opt_nonnull_to_ptr;
use crate::{Event, Result, Status, StatusExt};
use core::ptr::NonNull;
use core::sync::atomic::{AtomicUsize, Ordering};

pub use uefi_raw::protocol::block::{BlockIo2Protocol, BlockIoProtocol, Lba};

/// Block I/O [`Protocol`].
///
/// [`Protocol`]: uefi::proto::Protocol
#[derive(Debug)]
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
    /// * `extended_verification` Indicates that the driver may perform a more
    ///   exhaustive verification operation of the device during reset.
    ///
    /// # Errors
    /// * `Status::DEVICE_ERROR`  The block device is not functioning
    ///   correctly and could not be reset.
    pub fn reset(&mut self, extended_verification: bool) -> Result {
        unsafe { (self.0.reset)(&mut self.0, extended_verification.into()) }.to_result()
    }

    /// Read the requested number of blocks from the device.
    ///
    /// # Arguments
    /// * `media_id` - The media ID that the read request is for.
    /// * `lba` - The starting logical block address to read from on the device.
    /// * `buffer` - The target buffer of the read operation
    ///
    /// # Errors
    /// * `Status::DEVICE_ERROR`       The device reported an error while attempting to perform the read
    ///   operation.
    /// * `Status::NO_MEDIA`           There is no media in the device.
    /// * `Status::MEDIA_CHANGED`      The `media_id` is not for the current media.
    /// * `Status::BAD_BUFFER_SIZE`    The buffer size parameter is not a multiple of the intrinsic block size of
    ///   the device.
    /// * `Status::INVALID_PARAMETER`  The read request contains LBAs that are not valid, or the buffer is not on
    ///   proper alignment.
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
    /// * `Status::WRITE_PROTECTED`       The device cannot be written to.
    /// * `Status::NO_MEDIA`              There is no media in the device.
    /// * `Status::MEDIA_CHANGED`         The `media_id` is not for the current media.
    /// * `Status::DEVICE_ERROR`          The device reported an error while attempting to perform the write
    ///   operation.
    /// * `Status::BAD_BUFFER_SIZE`       The buffer size parameter is not a multiple of the intrinsic block size
    ///   of the device.
    /// * `Status::INVALID_PARAMETER`     The write request contains LBAs that are not valid, or the buffer is not
    ///   on proper alignment.
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
    /// * `Status::DEVICE_ERROR`          The device reported an error while attempting to write data.
    /// * `Status::NO_MEDIA`              There is no media in the device.
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
    pub fn is_removable_media(&self) -> bool {
        self.0.removable_media.into()
    }

    /// True if there is a media currently present in the device.
    #[must_use]
    pub fn is_media_present(&self) -> bool {
        self.0.media_present.into()
    }

    /// True if block IO was produced to abstract partition structure.
    #[must_use]
    pub fn is_logical_partition(&self) -> bool {
        self.0.logical_partition.into()
    }

    /// True if the media is marked read-only.
    #[must_use]
    pub fn is_read_only(&self) -> bool {
        self.0.read_only.into()
    }

    /// True if `writeBlocks` function writes data.
    #[must_use]
    pub fn is_write_caching(&self) -> bool {
        self.0.write_caching.into()
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

/// Asynchronous transaction token for Block I/O 2 operations.
#[repr(C)]
#[derive(Debug)]
pub struct BlockIO2Token {
    /// Event to be signalled when an asynchronous block I/O operation
    /// completes.
    pub event: Event,
    /// Transaction status code.
    // UEFI can change this at any time, so we need atomic access.
    pub transaction_status: AtomicUsize,
}

impl BlockIO2Token {
    /// Creates a new token.
    #[must_use]
    pub const fn new(event: Event, status: Status) -> Self {
        Self {
            event,
            transaction_status: AtomicUsize::new(status.0),
        }
    }

    /// Returns the transaction current status.
    pub fn transaction_status(&self) -> Status {
        Status(self.transaction_status.load(Ordering::SeqCst))
    }

    /// Clone this token.
    ///
    /// # Safety
    /// The caller must ensure that any clones of a closed `Event` are never
    /// used again.
    #[must_use]
    pub unsafe fn unsafe_clone(&self) -> Self {
        Self {
            event: unsafe { self.event.unsafe_clone() },
            transaction_status: AtomicUsize::new(self.transaction_status.load(Ordering::SeqCst)),
        }
    }
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
    /// Pointer for block IO media.
    #[must_use]
    pub const fn media(&self) -> &BlockIOMedia {
        unsafe { &*self.0.media.cast::<BlockIOMedia>() }
    }

    /// Resets the block device hardware.
    ///
    /// # Arguments
    /// * `extended_verification` - Indicates that the driver may perform a more exhaustive verification operation of the device during reset.
    ///
    /// # Errors
    /// * [`Status::DEVICE_ERROR`] The block device is not functioning correctly and could not be reset.
    pub fn reset(&mut self, extended_verification: bool) -> Result {
        unsafe { (self.0.reset)(&mut self.0, extended_verification.into()) }.to_result()
    }

    /// Reads the requested number of blocks from the device.
    ///
    /// # Arguments
    /// * `media_id` - The media ID that the read request is for.
    /// * `lba` - The starting logical block address to read from on the device.
    /// * `token` - Transaction token for asynchronous read or `None` for
    ///   synchronous operation.
    /// * `len` - Buffer size.
    /// * `buffer` - The target buffer of the read operation
    ///
    /// # Safety
    /// Because of the asynchronous nature of the block transaction, manual lifetime
    /// tracking is required.
    ///
    /// # Errors
    /// * [`Status::INVALID_PARAMETER`] The read request contains LBAs that are not valid, or the buffer is not on proper alignment.
    /// * [`Status::OUT_OF_RESOURCES`]  The request could not be completed due to a lack of resources.
    /// * [`Status::MEDIA_CHANGED`]     The `media_id` is not for the current media.
    /// * [`Status::NO_MEDIA`]          There is no media in the device.
    /// * [`Status::DEVICE_ERROR`]      The device reported an error while performing the read operation.
    /// * [`Status::BAD_BUFFER_SIZE`]   The buffer size parameter is not a multiple of the intrinsic block size of the device.
    pub unsafe fn read_blocks_ex(
        &self,
        media_id: u32,
        lba: Lba,
        token: Option<NonNull<BlockIO2Token>>,
        len: usize,
        buffer: *mut u8,
    ) -> Result {
        let token = opt_nonnull_to_ptr(token);
        unsafe { (self.0.read_blocks_ex)(&self.0, media_id, lba, token.cast(), len, buffer.cast()) }
            .to_result()
    }

    /// Writes a specified number of blocks to the device.
    ///
    /// # Arguments
    /// * `media_id` - The media ID that the write request is for.
    /// * `lba` - The starting logical block address to be written.
    /// * `token` - Transaction token for asynchronous write or `None` for
    ///   synchronous operation
    /// * `len` - Buffer size.
    /// * `buffer` - Buffer to be written from.
    ///
    /// # Safety
    /// Because of the asynchronous nature of the block transaction, manual
    /// lifetime tracking is required.
    ///
    /// # Errors
    /// * [`Status::INVALID_PARAMETER`] The write request contains LBAs that are not valid, or the buffer is not on proper alignment.
    /// * [`Status::OUT_OF_RESOURCES`]  The request could not be completed due to a lack of resources.
    /// * [`Status::MEDIA_CHANGED`]     The `media_id` is not for the current media.
    /// * [`Status::NO_MEDIA`]          There is no media in the device.
    /// * [`Status::DEVICE_ERROR`]      The device reported an error while performing the write operation.
    /// * [`Status::WRITE_PROTECTED`]   The device cannot be written to.
    /// * [`Status::BAD_BUFFER_SIZE`]   The buffer size parameter is not a multiple of the intrinsic block size of the device.
    pub unsafe fn write_blocks_ex(
        &mut self,
        media_id: u32,
        lba: Lba,
        token: Option<NonNull<BlockIO2Token>>,
        len: usize,
        buffer: *const u8,
    ) -> Result {
        let token = opt_nonnull_to_ptr(token);
        unsafe {
            (self.0.write_blocks_ex)(&mut self.0, media_id, lba, token.cast(), len, buffer.cast())
        }
        .to_result()
    }

    /// Flushes all modified data to the physical device.
    ///
    /// # Arguments
    /// * `token` - Transaction token for asynchronous flush.
    ///
    /// # Errors
    /// * [`Status::OUT_OF_RESOURCES`]  The request could not be completed due to a lack of resources.
    /// * [`Status::MEDIA_CHANGED`]     The media in the device has changed since the last access.
    /// * [`Status::NO_MEDIA`]          There is no media in the device.
    /// * [`Status::DEVICE_ERROR`]      The `media_id` is not for the current media.
    /// * [`Status::WRITE_PROTECTED`]   The device cannot be written to.
    pub fn flush_blocks_ex(&mut self, token: Option<NonNull<BlockIO2Token>>) -> Result {
        let token = opt_nonnull_to_ptr(token);
        unsafe { (self.0.flush_blocks_ex)(&mut self.0, token.cast()) }.to_result()
    }
}
