// SPDX-License-Identifier: MIT OR Apache-2.0

//! Disk I/O protocols [`DiskIo`] and [`DiskIo2`].

use crate::proto::unsafe_protocol;
use crate::util::opt_nonnull_to_ptr;
use crate::{Event, Result, Status, StatusExt};
use core::ptr::NonNull;
use uefi_raw::protocol::disk::{DiskIo2Protocol, DiskIoProtocol};

/// Disk I/O [`Protocol`].
///
/// This protocol is used to abstract the block accesses of the block I/O
/// protocol to a more general offset-length protocol. Firmware is
/// responsible for adding this protocol to any block I/O interface that
/// appears in the system that does not already have a disk I/O protocol.
///
/// [`Protocol`]: uefi::proto::Protocol
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(DiskIoProtocol::GUID)]
pub struct DiskIo(DiskIoProtocol);

impl DiskIo {
    /// Reads bytes from the disk device.
    ///
    /// # Arguments
    ///
    /// - `media_id`: Identifier of the medium to read.
    /// - `offset`: Byte offset at which to begin reading.
    /// - `buffer`: Destination buffer.
    ///
    /// # Errors
    ///
    /// - [`Status::INVALID_PARAMETER`]: the requested range is invalid.
    /// - [`Status::DEVICE_ERROR`]: the device reported an error.
    /// - [`Status::NO_MEDIA`]: no medium is present.
    /// - [`Status::MEDIA_CHANGED`]: `media_id` is not current.
    pub fn read_disk(&self, media_id: u32, offset: u64, buffer: &mut [u8]) -> Result {
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.read_disk)(
                &self.0,
                media_id,
                offset,
                buffer.len(),
                buffer.as_mut_ptr().cast(),
            )
        }
        .to_result()
    }

    /// Writes bytes to the disk device.
    ///
    /// # Arguments
    ///
    /// - `media_id`: Identifier of the medium to write.
    /// - `offset`: Byte offset at which to begin writing.
    /// - `buffer`: Source buffer.
    ///
    /// # Errors
    ///
    /// - [`Status::INVALID_PARAMETER`]: the requested range is invalid.
    /// - [`Status::DEVICE_ERROR`]: the device reported an error.
    /// - [`Status::NO_MEDIA`]: no medium is present.
    /// - [`Status::MEDIA_CHANGED`]: `media_id` is not current.
    /// - [`Status::WRITE_PROTECTED`]: the device is read-only.
    pub fn write_disk(&mut self, media_id: u32, offset: u64, buffer: &[u8]) -> Result {
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.write_disk)(
                &mut self.0,
                media_id,
                offset,
                buffer.len(),
                buffer.as_ptr().cast(),
            )
        }
        .to_result()
    }
}

/// Asynchronous transaction token for disk I/O 2 operations.
#[repr(C)]
#[derive(Debug)]
pub struct DiskIo2Token {
    /// Event to be signalled when an asynchronous disk I/O operation completes.
    pub event: Option<Event>,
    /// Transaction status code.
    pub transaction_status: Status,
}

/// Disk I/O 2 [`Protocol`].
///
/// This protocol provides an extension to the disk I/O protocol to enable
/// non-blocking / asynchronous byte-oriented disk operation.
///
/// [`Protocol`]: uefi::proto::Protocol
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(DiskIo2Protocol::GUID)]
pub struct DiskIo2(DiskIo2Protocol);

impl DiskIo2 {
    /// Terminates outstanding asynchronous requests to the device.
    ///
    /// # Errors
    ///
    /// - [`Status::DEVICE_ERROR`]: the device reported an error while
    ///   cancelling requests.
    pub fn cancel(&mut self) -> Result {
        // SAFETY: The memory is valid.
        unsafe { (self.0.cancel)(&mut self.0) }.to_result()
    }

    /// Reads bytes from the disk device.
    ///
    /// # Arguments
    ///
    /// - `media_id`: Identifier of the medium to read.
    /// - `offset`: Byte offset at which to begin reading.
    /// - `token`: Optional transaction token for asynchronous completion.
    /// - `len`: Number of bytes available at `buffer`.
    /// - `buffer`: Destination buffer.
    ///
    /// # Safety
    ///
    /// `token` and `buffer` must remain valid until the transaction completes.
    ///
    /// # Errors
    ///
    /// - [`Status::INVALID_PARAMETER`]: the requested range is invalid.
    /// - [`Status::OUT_OF_RESOURCES`]: insufficient resources for the request.
    /// - [`Status::MEDIA_CHANGED`]: `media_id` is not current.
    /// - [`Status::NO_MEDIA`]: no medium is present.
    /// - [`Status::DEVICE_ERROR`]: the device reported an error.
    pub unsafe fn read_disk_raw(
        &self,
        media_id: u32,
        offset: u64,
        token: Option<NonNull<DiskIo2Token>>,
        len: usize,
        buffer: *mut u8,
    ) -> Result {
        let token = opt_nonnull_to_ptr(token);
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.read_disk_ex)(&self.0, media_id, offset, token.cast(), len, buffer.cast())
        }
        .to_result()
    }

    /// Writes bytes to the disk device.
    ///
    /// # Arguments
    ///
    /// - `media_id`: Identifier of the medium to write.
    /// - `offset`: Byte offset at which to begin writing.
    /// - `token`: Optional transaction token for asynchronous completion.
    /// - `len`: Number of bytes available at `buffer`.
    /// - `buffer`: Source buffer.
    ///
    /// # Safety
    ///
    /// `token` and `buffer` must remain valid until the transaction completes.
    ///
    /// # Errors
    ///
    /// - [`Status::INVALID_PARAMETER`]: the requested range is invalid.
    /// - [`Status::OUT_OF_RESOURCES`]: insufficient resources for the request.
    /// - [`Status::MEDIA_CHANGED`]: `media_id` is not current.
    /// - [`Status::NO_MEDIA`]: no medium is present.
    /// - [`Status::DEVICE_ERROR`]: the device reported an error.
    /// - [`Status::WRITE_PROTECTED`]: the device is read-only.
    pub unsafe fn write_disk_raw(
        &mut self,
        media_id: u32,
        offset: u64,
        token: Option<NonNull<DiskIo2Token>>,
        len: usize,
        buffer: *const u8,
    ) -> Result {
        let token = opt_nonnull_to_ptr(token);
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.write_disk_ex)(
                &mut self.0,
                media_id,
                offset,
                token.cast(),
                len,
                buffer.cast(),
            )
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
    pub fn flush_disk(&mut self, token: Option<NonNull<DiskIo2Token>>) -> Result {
        let token = opt_nonnull_to_ptr(token);
        // SAFETY: The memory is valid.
        unsafe { (self.0.flush_disk_ex)(&mut self.0, token.cast()) }.to_result()
    }
}
