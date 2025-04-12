// SPDX-License-Identifier: MIT OR Apache-2.0

//! ATA Pass Thru Protocol.

use super::{AtaRequest, AtaResponse};
use crate::mem::{AlignedBuffer, PoolAllocation};
use crate::proto::device_path::PoolDevicePathNode;
use crate::StatusExt;
use core::alloc::LayoutError;
use core::ptr::{self, NonNull};
use uefi_macros::unsafe_protocol;
use uefi_raw::protocol::ata::AtaPassThruProtocol;
use uefi_raw::protocol::device_path::DevicePathProtocol;
use uefi_raw::Status;

/// Mode structure with controller-specific information.
pub type AtaPassThruMode = uefi_raw::protocol::ata::AtaPassThruMode;

/// The ATA Pass Thru Protocol.
///
/// One protocol instance represents one ATA controller connected to the machine.
///
/// This API offers a safe and convenient, yet still low-level interface to ATA devices.
/// It is designed as a foundational layer, leaving higher-level abstractions responsible for implementing
/// richer storage semantics, device-specific commands, and advanced use cases.
///
/// # UEFI Spec Description
/// Provides services that allow ATA commands to be sent to ATA Devices attached to an ATA controller. Packet-
/// based commands would be sent to ATAPI devices only through the Extended SCSI Pass Thru Protocol. While
/// the ATA_PASS_THRU interface would expose an interface to the underlying ATA devices on an ATA controller,
/// EXT_SCSI_PASS_THRU is responsible for exposing a packet-based command interface for the ATAPI devices on
/// the same ATA controller.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(AtaPassThruProtocol::GUID)]
pub struct AtaPassThru(AtaPassThruProtocol);

impl AtaPassThru {
    /// Retrieves the mode structure for the Extended SCSI Pass Thru protocol.
    ///
    /// # Returns
    /// The [`AtaPassThruMode`] structure containing configuration details of the protocol.
    #[must_use]
    pub fn mode(&self) -> AtaPassThruMode {
        unsafe { (*self.0.mode).clone() }
    }

    /// Retrieves the I/O buffer alignment required by this SCSI channel.
    ///
    /// # Returns
    /// - A `u32` value representing the required I/O alignment in bytes.
    #[must_use]
    pub fn io_align(&self) -> u32 {
        self.mode().io_align
    }

    /// Allocates an I/O buffer with the necessary alignment for this ATA Controller.
    ///
    /// You can alternatively do this yourself using the [`AlignedBuffer`] helper directly.
    /// The `ata` api will validate that your buffers have the correct alignment and error
    /// if they don't.
    ///
    /// # Parameters
    /// - `len`: The size (in bytes) of the buffer to allocate.
    ///
    /// # Returns
    /// [`AlignedBuffer`] containing the allocated memory.
    ///
    /// # Errors
    /// This method can fail due to alignment or memory allocation issues.
    pub fn alloc_io_buffer(&self, len: usize) -> Result<AlignedBuffer, LayoutError> {
        AlignedBuffer::from_size_align(len, self.io_align() as usize)
    }

    /// Iterate over all potential ATA devices on this channel.
    ///
    /// # Warning
    /// Depending on the UEFI implementation, this does not only return all actually available devices.
    /// Most implementations instead return a list of all possible fully-qualified device addresses.
    /// You have to probe for availability yourself, using [`AtaDevice::execute_command`].
    ///
    /// # Returns
    /// [`AtaDeviceIterator`] to iterate through connected ATA devices.
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
/// # Warning
/// This is only a potentially valid device address. Verify it by probing for an actually
/// available / connected device using [`AtaDevice::execute_command`] before doing anything meaningful.
#[derive(Debug)]
pub struct AtaDevice<'a> {
    proto: &'a AtaPassThruProtocol,
    port: u16,
    pmp: u16,
}

impl AtaDevice<'_> {
    fn proto_mut(&mut self) -> *mut AtaPassThruProtocol {
        ptr::from_ref(self.proto).cast_mut()
    }

    /// Returns the port number of the device.
    ///
    /// # Details
    /// - For SATA: This is the port number on the motherboard or controller.
    /// - For IDE: This is `0` for the primary bus and `1` for the secondary bus.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Returns the port multiplier port (PMP) number for the device.
    ///
    /// # Details
    /// - For SATA: `0xFFFF` indicates a direct connection to the port, while other values
    ///   indicate the port number on a port-multiplier device.
    /// - For IDE: `0` represents the master device, and `1` represents the slave device.
    #[must_use]
    pub const fn port_multiplier_port(&self) -> u16 {
        self.pmp
    }

    /// Resets the ATA device.
    ///
    /// This method attempts to reset the specified ATA device, restoring it to its default state.
    ///
    /// # Errors
    /// - [`Status::UNSUPPORTED`] The ATA controller does not support a device reset operation.
    /// - [`Status::INVALID_PARAMETER`] The `Port` or `PortMultiplierPort` values are invalid.
    /// - [`Status::DEVICE_ERROR`] A device error occurred while attempting to reset the specified ATA device.
    /// - [`Status::TIMEOUT`] A timeout occurred while attempting to reset the specified ATA device.
    pub fn reset(&mut self) -> crate::Result<()> {
        unsafe { (self.proto.reset_device)(self.proto_mut(), self.port, self.pmp).to_result() }
    }

    /// Get the final device path node for this device.
    ///
    /// For a full [`crate::proto::device_path::DevicePath`] pointing to this device, this needs to be appended to
    /// the controller's device path.
    pub fn path_node(&self) -> crate::Result<PoolDevicePathNode> {
        unsafe {
            let mut path_ptr: *const DevicePathProtocol = ptr::null();
            (self.proto.build_device_path)(self.proto, self.port, self.pmp, &mut path_ptr)
                .to_result()?;
            NonNull::new(path_ptr.cast_mut())
                .map(|p| PoolDevicePathNode(PoolAllocation::new(p.cast())))
                .ok_or(Status::OUT_OF_RESOURCES.into())
        }
    }

    /// Executes a command on the device.
    ///
    /// # Parameters
    /// - `req`: The request structure containing details about the command to execute.
    ///
    /// # Returns
    /// [`AtaResponse`] containing the results of the operation, such as data and status.
    ///
    /// # Errors
    /// - [`Status::BAD_BUFFER_SIZE`] The ATA command was not executed because the buffer size exceeded the allowed transfer size.
    ///   The number of bytes that could be transferred is returned in `InTransferLength` or `OutTransferLength`.
    /// - [`Status::NOT_READY`] The ATA command could not be sent because too many commands are already queued. Retry the operation later.
    /// - [`Status::DEVICE_ERROR`] A device error occurred while attempting to send the ATA command. Refer to `Asb` for additional status details.
    /// - [`Status::INVALID_PARAMETER`] The `Port`, `PortMultiplierPort`, or the contents of `Acb` are invalid.
    ///   The command was not sent, and no additional status information is available.
    /// - [`Status::UNSUPPORTED`] The host adapter does not support the command described by the ATA command.
    ///   The command was not sent, and no additional status information is available.
    /// - [`Status::TIMEOUT`] A timeout occurred while waiting for the ATA command to execute. Refer to `Asb` for additional status details.
    pub fn execute_command<'req>(
        &mut self,
        mut req: AtaRequest<'req>,
    ) -> crate::Result<AtaResponse<'req>> {
        req.packet.acb = &req.acb;
        unsafe {
            (self.proto.pass_thru)(
                self.proto_mut(),
                self.port,
                self.pmp,
                &mut req.packet,
                ptr::null_mut(),
            )
            .to_result_with_val(|| AtaResponse { req })
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
    proto: &'a AtaPassThruProtocol,
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
                let result = unsafe { (self.proto.get_next_port)(self.proto, &mut self.prev_port) };
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
            let result = unsafe {
                (self.proto.get_next_device)(self.proto, self.prev_port, &mut self.prev_pmp)
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
