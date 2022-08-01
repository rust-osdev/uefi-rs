//! Disk I/O protocols.

use crate::proto::Protocol;
use crate::{unsafe_guid, Event, Result, Status};
use core::ptr::NonNull;

/// The disk I/O protocol.
///
/// This protocol is used to abstract the block accesses of the block I/O
/// protocol to a more general offset-length protocol. Firmware is
/// reponsible for adding this protocol to any block I/O interface that
/// appears in the system that does not already have a disk I/O protocol.
#[repr(C)]
#[unsafe_guid("ce345171-ba0b-11d2-8e4f-00a0c969723b")]
#[derive(Protocol)]
pub struct DiskIo {
    revision: u64,
    read_disk: extern "efiapi" fn(
        this: &DiskIo,
        media_id: u32,
        offset: u64,
        len: usize,
        buffer: *mut u8,
    ) -> Status,
    write_disk: extern "efiapi" fn(
        this: &mut DiskIo,
        media_id: u32,
        offset: u64,
        len: usize,
        buffer: *const u8,
    ) -> Status,
}

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
        (self.read_disk)(self, media_id, offset, buffer.len(), buffer.as_mut_ptr()).into()
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
        (self.write_disk)(self, media_id, offset, buffer.len(), buffer.as_ptr()).into()
    }
}

/// Asynchronous transaction token for disk I/O 2 operations.
#[repr(C)]
pub struct DiskIo2Token {
    /// Event to be signalled when an asynchronous disk I/O operation completes.
    pub event: Event,
    /// Transaction status code.
    pub transaction_status: Status,
}

/// The disk I/O 2 protocol.
///
/// This protocol provides an extension to the disk I/O protocol to enable
/// non-blocking / asynchronous byte-oriented disk operation.
#[repr(C)]
#[unsafe_guid("151c8eae-7f2c-472c-9e54-9828194f6a88")]
#[derive(Protocol)]
pub struct DiskIo2 {
    revision: u64,
    cancel: extern "efiapi" fn(this: &mut DiskIo2) -> Status,
    read_disk_ex: extern "efiapi" fn(
        this: &DiskIo2,
        media_id: u32,
        offset: u64,
        token: Option<NonNull<DiskIo2Token>>,
        len: usize,
        buffer: *mut u8,
    ) -> Status,
    write_disk_ex: extern "efiapi" fn(
        this: &mut DiskIo2,
        media_id: u32,
        offset: u64,
        token: Option<NonNull<DiskIo2Token>>,
        len: usize,
        buffer: *const u8,
    ) -> Status,
    flush_disk_ex:
        extern "efiapi" fn(this: &mut DiskIo2, token: Option<NonNull<DiskIo2Token>>) -> Status,
}

impl DiskIo2 {
    /// Terminates outstanding asynchronous requests to the device.
    ///
    /// # Errors:
    /// * `uefi::status::DEVICE_ERROR`  The device reported an error while performing
    ///                                 the cancel operation.
    pub fn cancel(&mut self) -> Result {
        (self.cancel)(self).into()
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
        (self.read_disk_ex)(self, media_id, offset, token, len, buffer).into()
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
        (self.write_disk_ex)(self, media_id, offset, token, len, buffer).into()
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
        (self.flush_disk_ex)(self, token).into()
    }
}
