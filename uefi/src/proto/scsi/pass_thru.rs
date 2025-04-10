// SPDX-License-Identifier: MIT OR Apache-2.0

//! Extended SCSI Pass Thru protocols.

use super::{ScsiRequest, ScsiResponse};
use crate::mem::{AlignedBuffer, PoolAllocation};
use crate::proto::device_path::PoolDevicePathNode;
use crate::proto::unsafe_protocol;
use crate::StatusExt;
use core::alloc::LayoutError;
use core::ptr::{self, NonNull};
use uefi_raw::protocol::device_path::DevicePathProtocol;
use uefi_raw::protocol::scsi::{
    ExtScsiPassThruMode, ExtScsiPassThruProtocol, SCSI_TARGET_MAX_BYTES,
};
use uefi_raw::Status;

/// Structure representing a SCSI target address.
pub type ScsiTarget = [u8; SCSI_TARGET_MAX_BYTES];

/// Structure representing a fully-qualified device address, consisting of SCSI target and LUN.
#[derive(Clone, Debug)]
pub struct ScsiTargetLun(ScsiTarget, u64);
impl Default for ScsiTargetLun {
    fn default() -> Self {
        Self([0xFF; SCSI_TARGET_MAX_BYTES], 0)
    }
}

/// Enables interaction with SCSI devices using the Extended SCSI Pass Thru protocol.
///
/// This protocol allows communication with SCSI devices connected to the system,
/// providing methods to send commands, reset devices, and enumerate SCSI targets.
///
/// This API offers a safe and convenient, yet still low-level interface to SCSI devices.
/// It is designed as a foundational layer, leaving higher-level abstractions responsible for implementing
/// richer storage semantics, device-specific commands, and advanced use cases.
///
/// # UEFI Spec Description
/// Provides services that allow SCSI Pass Thru commands to be sent to SCSI devices attached to a SCSI channel. It also
/// allows packet-based commands (ATAPI cmds) to be sent to ATAPI devices attached to a ATA controller.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(ExtScsiPassThruProtocol::GUID)]
pub struct ExtScsiPassThru(ExtScsiPassThruProtocol);

impl ExtScsiPassThru {
    /// Retrieves the mode structure for the Extended SCSI Pass Thru protocol.
    ///
    /// # Returns
    /// The [`ExtScsiPassThruMode`] structure containing configuration details of the protocol.
    #[must_use]
    pub fn mode(&self) -> ExtScsiPassThruMode {
        let mut mode = unsafe { (*self.0.passthru_mode).clone() };
        mode.io_align = mode.io_align.max(1); // 0 and 1 is the same, says UEFI spec
        mode
    }

    /// Retrieves the I/O buffer alignment required by this SCSI channel.
    ///
    /// # Returns
    /// - A `u32` value representing the required I/O alignment.
    #[must_use]
    pub fn io_align(&self) -> u32 {
        self.mode().io_align
    }

    /// Allocates an I/O buffer with the necessary alignment for this SCSI channel.
    ///
    /// You can alternatively do this yourself using the [`AlignedBuffer`] helper directly.
    /// The Scsi api will validate that your buffers have the correct alignment and crash
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

    /// Iterate over all potential SCSI devices on this channel.
    ///
    /// # Warning
    /// Depending on the UEFI implementation, this does not only return all actually available devices.
    /// Most implementations instead return a list of all possible fully-qualified device addresses.
    /// You have to probe for availability yourself, using [`ScsiDevice::execute_command`].
    ///
    /// # Returns
    /// [`ScsiTargetLunIterator`] to iterate through connected SCSI devices.
    #[must_use]
    pub fn iter_devices(&self) -> ScsiTargetLunIterator<'_> {
        ScsiTargetLunIterator {
            proto: &self.0,
            prev: ScsiTargetLun::default(),
        }
    }

    /// Resets the SCSI channel associated with the protocol.
    ///
    /// The EFI_EXT_SCSI_PASS_THRU_PROTOCOL.ResetChannel() function resets a SCSI channel.
    /// This operation resets all the SCSI devices connected to the SCSI channel.
    ///
    /// # Returns
    /// [`Result<()>`] indicating the success or failure of the operation.
    ///
    /// # Errors
    /// - [`Status::UNSUPPORTED`] The SCSI channel does not support a channel reset operation.
    /// - [`Status::DEVICE_ERROR`] A device error occurred while attempting to reset the SCSI channel.
    /// - [`Status::TIMEOUT`] A timeout occurred while attempting to reset the SCSI channel.
    pub fn reset_channel(&mut self) -> crate::Result<()> {
        unsafe { (self.0.reset_channel)(&mut self.0).to_result() }
    }
}

/// Structure representing a potential ScsiDevice.
///
/// In the UEFI Specification, this corresponds to a (SCSI target, LUN) tuple.
///
/// # Warning
/// This does not actually have to correspond to an actual device!
/// You have to probe for availability before doing anything meaningful with it.
#[derive(Clone, Debug)]
pub struct ScsiDevice<'a> {
    proto: &'a ExtScsiPassThruProtocol,
    target_lun: ScsiTargetLun,
}
impl ScsiDevice<'_> {
    fn proto_mut(&mut self) -> *mut ExtScsiPassThruProtocol {
        ptr::from_ref(self.proto).cast_mut()
    }

    /// Returns the SCSI target address of the potential device.
    #[must_use]
    pub const fn target(&self) -> &ScsiTarget {
        &self.target_lun.0
    }

    /// Returns the logical unit number (LUN) of the potential device.
    #[must_use]
    pub const fn lun(&self) -> u64 {
        self.target_lun.1
    }

    /// Get the final device path node for this device.
    ///
    /// For a full [`crate::proto::device_path::DevicePath`] pointing to this device, this needs to be appended to
    /// the controller's device path.
    pub fn path_node(&self) -> crate::Result<PoolDevicePathNode> {
        unsafe {
            let mut path_ptr: *const DevicePathProtocol = ptr::null();
            (self.proto.build_device_path)(
                self.proto,
                self.target().as_ptr(),
                self.lun(),
                &mut path_ptr,
            )
            .to_result()?;
            NonNull::new(path_ptr.cast_mut())
                .map(|p| PoolDevicePathNode(PoolAllocation::new(p.cast())))
                .ok_or(Status::OUT_OF_RESOURCES.into())
        }
    }

    /// Resets the potential SCSI device represented by this instance.
    ///
    /// The `EFI_EXT_SCSI_PASS_THRU_PROTOCOL.ResetTargetLun()` function resets the SCSI logical unit
    /// specified by `Target` and `Lun`. This allows for recovering a device that may be in an error state
    /// or requires reinitialization. The function behavior is dependent on the SCSI channel's capability
    /// to perform target resets.
    ///
    /// # Returns
    /// [`Result<()>`] indicating the success or failure of the operation.
    ///
    /// # Errors
    /// - [`Status::UNSUPPORTED`] The SCSI channel does not support a target reset operation.
    /// - [`Status::INVALID_PARAMETER`] The `Target` or `Lun` values are invalid.
    /// - [`Status::DEVICE_ERROR`] A device error occurred while attempting to reset the SCSI device
    ///   specified by `Target` and `Lun`.
    /// - [`Status::TIMEOUT`] A timeout occurred while attempting to reset the SCSI device specified
    ///   by `Target` and `Lun`.
    pub fn reset(&mut self) -> crate::Result<()> {
        unsafe {
            (self.proto.reset_target_lun)(self.proto_mut(), self.target_lun.0.as_ptr(), self.lun())
                .to_result()
        }
    }

    /// Sends a SCSI command to the potential target device and retrieves the response.
    ///
    /// This method sends a SCSI Request Packet to a SCSI device attached to the SCSI channel.
    /// It supports both blocking and nonblocking I/O. Blocking I/O is mandatory, while
    /// nonblocking I/O is optional and dependent on the driver's implementation.
    ///
    /// # Parameters
    /// - `scsi_req`: The [`ScsiRequest`] containing the command and data to send to the device.
    ///
    /// # Returns
    /// [`ScsiResponse`] containing the results of the operation, such as data and status.
    ///
    /// # Errors
    /// - [`Status::BAD_BUFFER_SIZE`] The SCSI Request Packet was not executed because the data
    ///   buffer size exceeded the allowed transfer size for a single command. The number of bytes
    ///   that could be transferred is returned in `InTransferLength` or `OutTransferLength`.
    /// - [`Status::NOT_READY`] The SCSI Request Packet could not be sent because too many packets
    ///   are already queued. The caller may retry later.
    /// - [`Status::DEVICE_ERROR`] A device error occurred while attempting to send the SCSI Request Packet.
    ///   Additional status information is available in `HostAdapterStatus`, `TargetStatus`, `SenseDataLength`,
    ///   and `SenseData`.
    /// - [`Status::INVALID_PARAMETER`] The `Target`, `Lun`, or the contents of `ScsiRequestPacket` are invalid.
    ///   The SCSI Request Packet was not sent, and no additional status information is available.
    /// - [`Status::UNSUPPORTED`] The command described by the SCSI Request Packet is not supported by the
    ///   host adapter, including unsupported bi-directional SCSI commands. The SCSI Request Packet was not
    ///   sent, and no additional status information is available.
    /// - [`Status::TIMEOUT`] A timeout occurred while executing the SCSI Request Packet. Additional status
    ///   information is available in `HostAdapterStatus`, `TargetStatus`, `SenseDataLength`, and `SenseData`.
    pub fn execute_command<'req>(
        &mut self,
        mut scsi_req: ScsiRequest<'req>,
    ) -> crate::Result<ScsiResponse<'req>> {
        unsafe {
            (self.proto.pass_thru)(
                self.proto_mut(),
                self.target_lun.0.as_ptr(),
                self.target_lun.1,
                &mut scsi_req.packet,
                ptr::null_mut(),
            )
            .to_result_with_val(|| ScsiResponse(scsi_req))
        }
    }
}

/// An iterator over SCSI devices available on the channel.
#[derive(Debug)]
pub struct ScsiTargetLunIterator<'a> {
    proto: &'a ExtScsiPassThruProtocol,
    prev: ScsiTargetLun,
}
impl<'a> Iterator for ScsiTargetLunIterator<'a> {
    type Item = ScsiDevice<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // get_next_target_lun() takes the target as a double ptr, meaning that the spec allows
        // the implementation to return us a new buffer (most impls don't actually seem to do though)
        let mut target: *mut u8 = self.prev.0.as_mut_ptr();
        let result =
            unsafe { (self.proto.get_next_target_lun)(self.proto, &mut target, &mut self.prev.1) };
        if target != self.prev.0.as_mut_ptr() {
            // impl has returned us a new pointer instead of writing in our buffer, copy back
            unsafe {
                target.copy_to(self.prev.0.as_mut_ptr(), SCSI_TARGET_MAX_BYTES);
            }
        }
        let scsi_device = ScsiDevice {
            proto: self.proto,
            target_lun: self.prev.clone(),
        };
        match result {
            Status::SUCCESS => Some(scsi_device),
            Status::NOT_FOUND => None,
            _ => panic!("Must not happen according to spec!"),
        }
    }
}
