//! Disk I/O protocols.

use crate::proto::unsafe_protocol;
use crate::util::opt_nonnull_to_ptr;
use crate::{Event, Result, Status, StatusExt};
use core::ptr::NonNull;
use uefi_raw::protocol::disk::{DiskIo2Protocol, DiskIoProtocol};

/// The disk I/O protocol.
///
/// This protocol is used to abstract the block accesses of the block I/O
/// protocol to a more general offset-length protocol. Firmware is
/// responsible for adding this protocol to any block I/O interface that
/// appears in the system that does not already have a disk I/O protocol.
#[repr(transparent)]
#[unsafe_protocol(DiskIoProtocol::GUID)]
pub struct DiskIo(DiskIoProtocol);

impl DiskIo {
    /// Reads bytes from the disk device.
    ///
    /// # Arguments:
    /// * `media_id` - ID of the medium to be read.
    /// * `offset` - Starting byte offset on the logical block I/O device to read from.
    /// * `buffer` - Pointer to a buffer to read into.
    ///
    /// # Errors:
    /// * `uefi::status::INVALID_PARAMETER` The read request contains device addresses that
    ///                                     are not valid for the device.
    /// * `uefi::status::DEVICE_ERROR`      The device reported an error while performing
    ///                                     the read operation.
    /// * `uefi::status::NO_MEDIA`          There is no medium in the device.
    /// * `uefi::status::MEDIA_CHANGED`     `media_id` is not for the current medium.
    pub fn read_disk(&self, media_id: u32, offset: u64, buffer: &mut [u8]) -> Result {
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
    /// # Arguments:
    /// * `media_id` - ID of the medium to be written.
    /// * `offset` - Starting byte offset on the logical block I/O device to write to.
    /// * `buffer` - Pointer to a buffer to write from.
    ///
    /// # Errors:
    /// * `uefi::status::INVALID_PARAMETER` The write request contains device addresses that
    ///                                     are not valid for the device.
    /// * `uefi::status::DEVICE_ERROR`      The device reported an error while performing
    ///                                     the write operation.
    /// * `uefi::status::NO_MEDIA`          There is no medium in the device.
    /// * `uefi::status::MEDIA_CHANGED`     `media_id` is not for the current medium.
    /// * `uefi::status::WRITE_PROTECTED`   The device cannot be written to.
    pub fn write_disk(&mut self, media_id: u32, offset: u64, buffer: &[u8]) -> Result {
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

/// The disk I/O 2 protocol.
///
/// This protocol provides an extension to the disk I/O protocol to enable
/// non-blocking / asynchronous byte-oriented disk operation.
#[repr(transparent)]
#[unsafe_protocol(DiskIo2Protocol::GUID)]
pub struct DiskIo2(DiskIo2Protocol);

impl DiskIo2 {
    /// Terminates outstanding asynchronous requests to the device.
    ///
    /// # Errors:
    /// * `uefi::status::DEVICE_ERROR`  The device reported an error while performing
    ///                                 the cancel operation.
    pub fn cancel(&mut self) -> Result {
        unsafe { (self.0.cancel)(&mut self.0) }.to_result()
    }

    /// Reads bytes from the disk device.
    ///
    /// # Arguments:
    /// * `media_id` - ID of the medium to be read from.
    /// * `offset` - Starting byte offset on the logical block I/O device to read from.
    /// * `token` - Transaction token for asynchronous read.
    /// * `len` - Buffer size.
    /// * `buffer` - Buffer to read into.
    ///
    /// # Safety
    ///
    /// Because of the asynchronous nature of the disk transaction, manual lifetime
    /// tracking is required.
    ///
    /// # Errors:
    /// * `uefi::status::INVALID_PARAMETER` The read request contains device addresses
    ///                                     that are not valid for the device.
    /// * `uefi::status::OUT_OF_RESOURCES`  The request could not be completed due to
    ///                                     a lack of resources.
    /// * `uefi::status::MEDIA_CHANGED`     `media_id` is not for the current medium.
    /// * `uefi::status::NO_MEDIA`          There is no medium in the device.
    /// * `uefi::status::DEVICE_ERROR`      The device reported an error while performing
    ///                                     the read operation.
    pub unsafe fn read_disk_raw(
        &self,
        media_id: u32,
        offset: u64,
        token: Option<NonNull<DiskIo2Token>>,
        len: usize,
        buffer: *mut u8,
    ) -> Result {
        let token = opt_nonnull_to_ptr(token);
        (self.0.read_disk_ex)(&self.0, media_id, offset, token.cast(), len, buffer.cast())
            .to_result()
    }

    /// Writes bytes to the disk device.
    ///
    /// # Arguments:
    /// * `media_id` - ID of the medium to write to.
    /// * `offset` - Starting byte offset on the logical block I/O device to write to.
    /// * `token` - Transaction token for asynchronous write.
    /// * `len` - Buffer size.
    /// * `buffer` - Buffer to write from.
    ///
    /// # Safety
    ///
    /// Because of the asynchronous nature of the disk transaction, manual lifetime
    /// tracking is required.
    ///
    /// # Errors:
    /// * `uefi::status::INVALID_PARAMETER` The write request contains device addresses
    ///                                     that are not valid for the device.
    /// * `uefi::status::OUT_OF_RESOURCES`  The request could not be completed due to
    ///                                     a lack of resources.
    /// * `uefi::status::MEDIA_CHANGED`     `media_id` is not for the current medium.
    /// * `uefi::status::NO_MEDIA`          There is no medium in the device.
    /// * `uefi::status::DEVICE_ERROR`      The device reported an error while performing
    ///                                     the write operation.
    /// * `uefi::status::WRITE_PROTECTED`   The device cannot be written to.
    pub unsafe fn write_disk_raw(
        &mut self,
        media_id: u32,
        offset: u64,
        token: Option<NonNull<DiskIo2Token>>,
        len: usize,
        buffer: *const u8,
    ) -> Result {
        let token = opt_nonnull_to_ptr(token);
        (self.0.write_disk_ex)(
            &mut self.0,
            media_id,
            offset,
            token.cast(),
            len,
            buffer.cast(),
        )
        .to_result()
    }

    /// Flushes all modified data to the physical device.
    ///
    /// # Arguments:
    /// * `token` - Transaction token for the asynchronous flush.
    ///
    /// # Errors:
    /// * `uefi::status::OUT_OF_RESOURCES`  The request could not be completed due to
    ///                                     a lack of resources.
    /// * `uefi::status::MEDIA_CHANGED`     The medium in the device has changed since
    ///                                     the last access.
    /// * `uefi::status::NO_MEDIA`          There is no medium in the device.
    /// * `uefi::status::DEVICE_ERROR`      The device reported an error while performing
    ///                                     the flush operation.
    /// * `uefi::status::WRITE_PROTECTED`   The device cannot be written to.
    pub fn flush_disk(&mut self, token: Option<NonNull<DiskIo2Token>>) -> Result {
        let token = opt_nonnull_to_ptr(token);
        unsafe { (self.0.flush_disk_ex)(&mut self.0, token.cast()) }.to_result()
    }
}
