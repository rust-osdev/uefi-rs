// SPDX-License-Identifier: MIT OR Apache-2.0

//! ATA Pass Thru protocol.

use super::{AtaRequest, AtaResponse};
use crate::StatusExt;
use crate::mem::{AlignedBuffer, PoolAllocation};
use crate::proto::device_path::PoolDevicePathNode;
use core::alloc::LayoutError;
use core::cell::UnsafeCell;
use core::ptr::{self, NonNull};
use uefi_macros::unsafe_protocol;
use uefi_raw::Status;
use uefi_raw::protocol::ata::AtaPassThruProtocol;
use uefi_raw::protocol::device_path::DevicePathProtocol;

/// Mode structure with controller-specific information.
pub type AtaPassThruMode = uefi_raw::protocol::ata::AtaPassThruMode;

/// The ATA Pass Thru Protocol.
///
/// One protocol instance represents one ATA controller connected to the machine.
///
/// This API offers a safe, convenient, but still low-level interface to ATA
/// devices. Higher-level abstractions remain responsible for storage
/// semantics and device-specific commands.
///
/// # UEFI Specification
/// Provides services that allow ATA commands to be sent to ATA Devices attached to an ATA controller. Packet-
/// based commands would be sent to ATAPI devices only through the Extended SCSI Pass Thru Protocol. While
/// the ATA_PASS_THRU interface would expose an interface to the underlying ATA devices on an ATA controller,
/// EXT_SCSI_PASS_THRU is responsible for exposing a packet-based command interface for the ATAPI devices on
/// the same ATA controller.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(AtaPassThruProtocol::GUID)]
pub struct AtaPassThru(UnsafeCell<AtaPassThruProtocol>);

impl AtaPassThru {
    /// Returns the ATA Pass Thru mode.
    #[must_use]
    pub fn mode(&self) -> AtaPassThruMode {
        // SAFETY: The memory is valid.
        let mut mode = unsafe { (*(*self.0.get()).mode).clone() };
        mode.io_align = mode.io_align.max(1); // 0 and 1 is the same, says UEFI spec
        mode
    }

    /// Returns the required I/O buffer alignment in bytes.
    #[must_use]
    pub fn io_align(&self) -> u32 {
        self.mode().io_align
    }

    /// Allocates an I/O buffer with the alignment required by this controller.
    ///
    /// Callers can instead allocate an [`AlignedBuffer`] directly. ATA request
    /// builders validate user-provided buffer alignment.
    ///
    /// # Arguments
    ///
    /// - `len`: Number of bytes to allocate.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer cannot be allocated with the required
    /// alignment.
    pub fn alloc_io_buffer(&self, len: usize) -> Result<AlignedBuffer, LayoutError> {
        AlignedBuffer::from_size_align(len, self.io_align() as usize)
    }

    /// Returns an iterator over all potential ATA devices on this channel.
    ///
    /// # Warnings
    ///
    /// Firmware may return every possible fully qualified device address, not
    /// only addresses with a connected device. Probe each address with
    /// [`AtaDevice::execute_command`].
    #[must_use]
    pub const fn iter_devices(&self) -> AtaDeviceIterator<'_> {
        AtaDeviceIterator {
            proto: &self.0,
            end_of_port: true,
            prev_port: 0xFFFF,
            prev_pmp: 0xFFFF,
        }
    }
}

/// Represents an ATA device on a controller.
///
/// # Warnings
///
/// This is only a potentially valid device address. Probe it with
/// [`AtaDevice::execute_command`] before use.
#[derive(Debug)]
pub struct AtaDevice<'a> {
    proto: &'a UnsafeCell<AtaPassThruProtocol>,
    port: u16,
    pmp: u16,
}

impl AtaDevice<'_> {
    /// Returns the port number of the device.
    ///
    /// For SATA, this is the port on the motherboard or controller. For IDE,
    /// `0` is the primary bus and `1` is the secondary bus.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Returns the port multiplier port (PMP) number for the device.
    ///
    /// For SATA, `0xFFFF` indicates a direct connection; other values identify
    /// a port on a port-multiplier device. For IDE, `0` is the master and `1`
    /// is the slave device.
    #[must_use]
    pub const fn port_multiplier_port(&self) -> u16 {
        self.pmp
    }

    /// Resets the ATA device.
    ///
    /// This restores the device to its default state.
    ///
    /// # Errors
    /// - [`Status::UNSUPPORTED`]: the controller does not support device reset.
    /// - [`Status::INVALID_PARAMETER`]: the port or PMP value is invalid.
    /// - [`Status::DEVICE_ERROR`]: the device reported an error.
    /// - [`Status::TIMEOUT`]: the reset timed out.
    pub fn reset(&mut self) -> crate::Result<()> {
        // SAFETY: The memory is valid.
        unsafe {
            ((*self.proto.get()).reset_device)(self.proto.get(), self.port, self.pmp).to_result()
        }
    }

    /// Returns the final device path node for this device.
    ///
    /// Append this node to the controller's device path to form a complete
    /// [`crate::proto::device_path::DevicePath`].
    ///
    /// # Errors
    ///
    /// Returns an error if firmware cannot build the node or allocation fails.
    pub fn path_node(&self) -> crate::Result<PoolDevicePathNode> {
        // SAFETY: The memory is valid.
        unsafe {
            let mut path_ptr: *const DevicePathProtocol = ptr::null();
            ((*self.proto.get()).build_device_path)(
                self.proto.get(),
                self.port,
                self.pmp,
                &mut path_ptr,
            )
            .to_result()?;
            NonNull::new(path_ptr.cast_mut())
                .map(|p| PoolDevicePathNode(PoolAllocation::new(p.cast())))
                .ok_or_else(|| Status::OUT_OF_RESOURCES.into())
        }
    }

    /// Executes a command on the device.
    ///
    /// On failure, the error contains an [`AtaResponse`] with any status and
    /// transfer information produced by the controller.
    ///
    /// # Errors
    ///
    /// - [`Status::BAD_BUFFER_SIZE`]: the buffer exceeds the allowed transfer
    ///   size.
    /// - [`Status::NOT_READY`]: too many commands are queued; retry later.
    /// - [`Status::DEVICE_ERROR`]: the device reported an error.
    /// - [`Status::INVALID_PARAMETER`]: the port, PMP, or command block is
    ///   invalid.
    /// - [`Status::UNSUPPORTED`]: the host adapter does not support the
    ///   command.
    /// - [`Status::TIMEOUT`]: the command timed out.
    #[expect(clippy::result_large_err)]
    pub fn execute_command<'req>(
        &mut self,
        mut req: AtaRequest<'req>,
    ) -> crate::Result<AtaResponse<'req>, AtaResponse<'req>> {
        req.packet.acb = &req.acb;
        // SAFETY: The memory is valid.
        let result = unsafe {
            ((*self.proto.get()).pass_thru)(
                self.proto.get(),
                self.port,
                self.pmp,
                &mut req.packet,
                ptr::null_mut(),
            )
            .to_result()
        };
        match result {
            Ok(_) => Ok(AtaResponse { req }),
            Err(s) => Err(crate::Error::new(s.status(), AtaResponse { req })),
        }
    }
}

/// An iterator over the drives connected to an ATA controller.
///
/// The iterator yields [`AtaDevice`] instances, each representing one *potential*
/// drive connected to the ATA controller. You have to probe whether the drive
/// is actually available and connected!
#[derive(Debug)]
pub struct AtaDeviceIterator<'a> {
    proto: &'a UnsafeCell<AtaPassThruProtocol>,
    // when there are no more devices on this port -> get next port
    end_of_port: bool,
    prev_port: u16,
    prev_pmp: u16,
}

impl<'a> Iterator for AtaDeviceIterator<'a> {
    type Item = AtaDevice<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.end_of_port {
                // SAFETY: The memory is valid.
                let result = unsafe {
                    ((*self.proto.get()).get_next_port)(self.proto.get(), &mut self.prev_port)
                };
                match result {
                    Status::SUCCESS => self.end_of_port = false,
                    Status::NOT_FOUND => return None, // no more ports / devices. End of list
                    _ => panic!("Must not happen according to spec!"),
                }
            }
            // get next device on port
            // The UEFI spec states, that:
            //   If there is no port multiplier detected on the given port, the initial query of get_next_device()
            //   is allowed to return either of:
            //      - EFI_SUCCESS & PMP = 0xFFFF
            //      - EFI_NOT_FOUND
            //   But even when there is no detected port multiplier, there might be a device directly connected
            //   to the port! A port where the device is directly connected uses a pmp-value of 0xFFFF.
            let was_first = self.prev_pmp == 0xFFFF;
            // SAFETY: The memory is valid.
            let result = unsafe {
                ((*self.proto.get()).get_next_device)(
                    self.proto.get(),
                    self.prev_port,
                    &mut self.prev_pmp,
                )
            };
            match result {
                Status::SUCCESS => {
                    if self.prev_pmp == 0xFFFF {
                        self.end_of_port = true;
                    }
                    return Some(AtaDevice {
                        proto: self.proto,
                        port: self.prev_port,
                        pmp: self.prev_pmp,
                    });
                }
                Status::NOT_FOUND => {
                    self.end_of_port = true;
                    self.prev_pmp = 0xFFFF;
                    if was_first {
                        // no port multiplier on port, return valid device anyway.
                        return Some(AtaDevice {
                            proto: self.proto,
                            port: self.prev_port,
                            pmp: 0xFFFF,
                        });
                    }
                }
                _ => panic!("Must not happen according to spec!"),
            }
        }
    }
}
