// SPDX-License-Identifier: MIT OR Apache-2.0

//! DiskInfo protocol.

use crate::StatusExt;
use uefi_macros::unsafe_protocol;
use uefi_raw::protocol::disk::DiskInfoProtocol;

#[cfg(doc)]
use crate::Status;

/// Interface used by a disk device.
///
/// [`DiskInfoInterface::Unknown`] represents an unrecognized or unsupported
/// interface.
#[derive(Debug, Eq, PartialEq)]
pub enum DiskInfoInterface {
    /// Unrecognized or unsupported interface.
    Unknown,
    /// Integrated Drive Electronics (IDE) interface.
    IDE,
    /// Universal Flash Storage (UFS) interface.
    UFS,
    /// Universal Serial Bus (USB) interface.
    USB,
    /// Advanced Host Controller Interface (AHCI) interface.
    AHCI,
    /// Non-Volatile Memory Express (NVME) interface.
    NVME,
    /// Small Computer System Interface (SCSI).
    SCSI,
    /// Secure Digital Memory Card (SDMMC) interface.
    SDMMC,
}

/// Metadata returned by [`DiskInfo::sense_data`].
#[derive(Debug)]
pub struct SenseDataInfo {
    /// Number of bytes returned by [`DiskInfo::sense_data`].
    pub bytes: usize,
    /// Number of sense-data records written to the result buffer.
    pub number: u8,
}

/// Physical location of a device on its bus.
///
/// This is not supported by all interface types.
#[derive(Debug)]
pub struct DeviceLocationInfo {
    /// For IDE, this addresses the channel (primary or secondary).
    /// For AHCI, this returns the port.
    pub channel: u32,
    /// For IDE, this contains whether the device is master or slave.
    /// For AHCI, this returns the port multiplier port.
    pub device: u32,
}

/// Disk Info [`Protocol`].
///
/// This allows querying hardware information for detected disks in a simple way.
/// Originally, this was designed for IDE and it shows.
/// But support for a wide range of interfaces was retrofitted.
///
/// Not all operations are supported by all interface types!
/// Either use [`DiskInfo::interface`] to determine what should be possible, or simply
/// try and handle the [`Status::UNSUPPORTED`] error return value.
///
/// # UEFI Specification
/// Provides the basic interfaces to abstract platform information regarding an IDE controller.
///
/// [`Protocol`]: uefi::proto::Protocol
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(DiskInfoProtocol::GUID)]
pub struct DiskInfo(DiskInfoProtocol);

impl DiskInfo {
    /// Returns the disk device's interface type.
    #[must_use]
    pub const fn interface(&self) -> DiskInfoInterface {
        match self.0.interface {
            DiskInfoProtocol::IDE_INTERFACE_GUID => DiskInfoInterface::IDE,
            DiskInfoProtocol::UFS_INTERFACE_GUID => DiskInfoInterface::UFS,
            DiskInfoProtocol::USB_INTERFACE_GUID => DiskInfoInterface::USB,
            DiskInfoProtocol::AHCI_INTERFACE_GUID => DiskInfoInterface::AHCI,
            DiskInfoProtocol::NVME_INTERFACE_GUID => DiskInfoInterface::NVME,
            DiskInfoProtocol::SCSI_INTERFACE_GUID => DiskInfoInterface::SCSI,
            DiskInfoProtocol::SD_MMC_INTERFACE_GUID => DiskInfoInterface::SDMMC,
            _ => DiskInfoInterface::Unknown,
        }
    }

    /// Performs an inquiry command on the disk device.
    ///
    /// # Arguments
    ///
    /// - `bfr`: Buffer in which to store the inquiry data.
    ///
    /// # Returns
    ///
    /// Returns the number of bytes written to `bfr`.
    ///
    /// # Errors
    /// - [`Status::NOT_FOUND`]: the device does not support inquiry data.
    /// - [`Status::DEVICE_ERROR`]: the device reported an error.
    /// - [`Status::BUFFER_TOO_SMALL`]: `bfr` is too small.
    pub fn inquiry(&self, bfr: &mut [u8]) -> crate::Result<usize> {
        let mut len: u32 = bfr.len() as u32;
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.inquiry)(&self.0, bfr.as_mut_ptr().cast(), &mut len)
                .to_result_with_val(|| len as usize)
        }
    }

    /// Performs an identify command on the disk device.
    ///
    /// # Arguments
    ///
    /// - `bfr`: Buffer in which to store the identification data.
    ///
    /// # Returns
    ///
    /// Returns the number of bytes written to `bfr`.
    ///
    /// # Errors
    /// - [`Status::NOT_FOUND`]: the device does not support identify data.
    /// - [`Status::DEVICE_ERROR`]: the device reported an error.
    /// - [`Status::BUFFER_TOO_SMALL`]: `bfr` is too small.
    pub fn identify(&self, bfr: &mut [u8]) -> crate::Result<usize> {
        let mut len: u32 = bfr.len() as u32;
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.identify)(&self.0, bfr.as_mut_ptr().cast(), &mut len)
                .to_result_with_val(|| len as usize)
        }
    }

    /// Retrieves sense data from the disk device.
    ///
    /// # Arguments
    ///
    /// - `bfr`: Buffer in which to store the sense data.
    ///
    /// # Returns
    ///
    /// Returns the byte count and number of sense-data records written.
    ///
    /// # Errors
    /// - [`Status::NOT_FOUND`]: the device does not support sense data.
    /// - [`Status::DEVICE_ERROR`]: the device reported an error.
    /// - [`Status::BUFFER_TOO_SMALL`]: `bfr` is too small.
    pub fn sense_data(&self, bfr: &mut [u8]) -> crate::Result<SenseDataInfo> {
        let mut len: u32 = bfr.len() as u32;
        let mut number: u8 = 0;
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.sense_data)(&self.0, bfr.as_mut_ptr().cast(), &mut len, &mut number)
                .to_result_with_val(|| SenseDataInfo {
                    bytes: len as usize,
                    number,
                })
        }
    }

    /// Retrieves the physical location of the device on the bus.
    ///
    /// This operation provides the channel and device identifiers that identify
    /// the device's physical connection point.
    ///
    /// # Errors
    /// - [`Status::UNSUPPORTED`]: the disk interface does not expose a bus
    ///   location.
    pub fn bus_location(&self) -> crate::Result<DeviceLocationInfo> {
        let mut ide_channel: u32 = 0; // called ide, but also useful for other interfaces
        let mut ide_device: u32 = 0;
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.which_ide)(&self.0, &mut ide_channel, &mut ide_device).to_result_with_val(
                || DeviceLocationInfo {
                    channel: ide_channel,
                    device: ide_device,
                },
            )
        }
    }
}
