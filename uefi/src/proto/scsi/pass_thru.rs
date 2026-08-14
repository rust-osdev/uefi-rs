// SPDX-License-Identifier: MIT OR Apache-2.0

//! Extended SCSI Pass Thru protocols.

use super::{ScsiRequest, ScsiResponse};
use crate::StatusExt;
use crate::mem::{AlignedBuffer, PoolAllocation};
use crate::proto::device_path::PoolDevicePathNode;
use crate::proto::unsafe_protocol;
use core::alloc::LayoutError;
use core::cell::UnsafeCell;
use core::ptr::{self, NonNull};
use uefi_raw::Status;
use uefi_raw::protocol::device_path::DevicePathProtocol;
use uefi_raw::protocol::scsi::{
    ExtScsiPassThruMode, ExtScsiPassThruProtocol, SCSI_TARGET_MAX_BYTES,
};

/// SCSI target address.
pub type ScsiTarget = [u8; SCSI_TARGET_MAX_BYTES];

/// Fully qualified SCSI target and logical unit number (LUN).
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
/// This is a safe, convenient, but still low-level interface. Higher-level
/// abstractions remain responsible for storage semantics and device-specific
/// commands.
///
/// # UEFI Specification
/// Provides services that allow SCSI Pass Thru commands to be sent to SCSI devices attached to a SCSI channel. It also
/// allows packet-based commands (ATAPI cmds) to be sent to ATAPI devices attached to a ATA controller.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(ExtScsiPassThruProtocol::GUID)]
pub struct ExtScsiPassThru(UnsafeCell<ExtScsiPassThruProtocol>);

impl ExtScsiPassThru {
    /// Returns the Extended SCSI Pass Thru mode.
    #[must_use]
    pub fn mode(&self) -> ExtScsiPassThruMode {
        // SAFETY: The memory is valid.
        let mut mode = unsafe { (*(*self.0.get()).passthru_mode).clone() };
        mode.io_align = mode.io_align.max(1); // 0 and 1 is the same, says UEFI spec
        mode
    }

    /// Returns the required I/O buffer alignment in bytes.
    #[must_use]
    pub fn io_align(&self) -> u32 {
        self.mode().io_align
    }

    /// Allocates an I/O buffer with the alignment required by this channel.
    ///
    /// Callers can instead allocate an [`AlignedBuffer`] directly. SCSI request
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

    /// Returns an iterator over all potential SCSI devices on this channel.
    ///
    /// # Warnings
    ///
    /// Firmware may return every possible fully qualified address, not only
    /// addresses with a connected device. Probe each address with
    /// [`ScsiDevice::execute_command`].
    #[must_use]
    pub fn iter_devices(&self) -> ScsiTargetLunIterator<'_> {
        ScsiTargetLunIterator {
            proto: &self.0,
            prev: ScsiTargetLun::default(),
        }
    }

    /// Resets the SCSI channel associated with the protocol.
    ///
    /// This also resets every SCSI device connected to the channel.
    ///
    /// # Errors
    /// - [`Status::UNSUPPORTED`]: the channel does not support reset.
    /// - [`Status::DEVICE_ERROR`]: a device reported an error.
    /// - [`Status::TIMEOUT`]: the reset timed out.
    pub fn reset_channel(&mut self) -> crate::Result<()> {
        // SAFETY: The memory is valid.
        unsafe { ((*self.0.get()).reset_channel)(self.0.get()).to_result() }
    }
}

/// Potential SCSI device identified by a target and LUN.
///
/// In the UEFI Specification, this corresponds to a (SCSI target, LUN) tuple.
///
/// # Warnings
///
/// This address need not correspond to a connected device. Probe it before
/// use.
#[derive(Clone, Debug)]
pub struct ScsiDevice<'a> {
    proto: &'a UnsafeCell<ExtScsiPassThruProtocol>,
    target_lun: ScsiTargetLun,
}
impl ScsiDevice<'_> {
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
                self.target().as_ptr(),
                self.lun(),
                &mut path_ptr,
            )
            .to_result()?;
            NonNull::new(path_ptr.cast_mut())
                .map(|p| PoolDevicePathNode(PoolAllocation::new(p.cast())))
                .ok_or_else(|| Status::OUT_OF_RESOURCES.into())
        }
    }

    /// Resets this potential SCSI device.
    ///
    /// This can recover a device that is in an error state or requires
    /// reinitialization, if the channel supports target resets.
    ///
    /// # Errors
    /// - [`Status::UNSUPPORTED`]: the channel does not support target reset.
    /// - [`Status::INVALID_PARAMETER`]: the target or LUN is invalid.
    /// - [`Status::DEVICE_ERROR`]: the device reported an error.
    /// - [`Status::TIMEOUT`]: the reset timed out.
    pub fn reset(&mut self) -> crate::Result<()> {
        // SAFETY: The memory is valid.
        unsafe {
            ((*self.proto.get()).reset_target_lun)(
                self.proto.get(),
                self.target_lun.0.as_ptr(),
                self.lun(),
            )
            .to_result()
        }
    }

    /// Sends a SCSI command to this potential target device.
    ///
    /// This wrapper performs blocking I/O.
    ///
    /// # Errors
    /// - [`Status::BAD_BUFFER_SIZE`]: a data buffer exceeds the allowed
    ///   transfer size.
    /// - [`Status::NOT_READY`]: too many requests are queued; retry later.
    /// - [`Status::DEVICE_ERROR`]: the device reported an error.
    /// - [`Status::INVALID_PARAMETER`]: the target, LUN, or request is invalid.
    /// - [`Status::UNSUPPORTED`]: the host adapter does not support the
    ///   command.
    /// - [`Status::TIMEOUT`]: the command timed out.
    pub fn execute_command<'req>(
        &mut self,
        mut scsi_req: ScsiRequest<'req>,
    ) -> crate::Result<ScsiResponse<'req>> {
        // SAFETY: The memory is valid.
        unsafe {
            ((*self.proto.get()).pass_thru)(
                self.proto.get(),
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
    proto: &'a UnsafeCell<ExtScsiPassThruProtocol>,
    prev: ScsiTargetLun,
}
impl<'a> Iterator for ScsiTargetLunIterator<'a> {
    type Item = ScsiDevice<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // get_next_target_lun() takes the target as a double ptr, meaning that the spec allows
        // the implementation to return us a new buffer (most impls don't actually seem to do though)
        let mut target: *mut u8 = self.prev.0.as_mut_ptr();
        // SAFETY: The memory is valid.
        let result = unsafe {
            ((*self.proto.get()).get_next_target_lun)(
                self.proto.get(),
                &mut target,
                &mut self.prev.1,
            )
        };
        if target != self.prev.0.as_mut_ptr() {
            // impl has returned us a new pointer instead of writing in our buffer, copy back
            // SAFETY: The memory is valid.
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
