// SPDX-License-Identifier: MIT OR Apache-2.0

//! DiskInfo protocol.

use crate::StatusExt;
use uefi_macros::unsafe_protocol;
use uefi_raw::protocol::disk::DiskInfoProtocol;

/// Enum representing the interface type of the disk.
///
/// This protocol abstracts various disk interfaces, including IDE, USB, AHCI, NVME, and more.
/// Unknown indicates an unrecognized or not yet implemented interface type.
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

/// Structure containing metadata about the result for a call to [`DiskInfo::sense_data`].
#[derive(Debug)]
pub struct SenseDataInfo {
    /// Amount of bytes returned by the [`DiskInfo::sense_data`].
    pub bytes: usize,
    /// Number of sense data messages contained in the resulting buffer from calling [`DiskInfo::sense_data`].
    pub number: u8,
}

/// Structure containing information about the physical device location on the bus.
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

/// DiskInfo protocol.
///
/// This allows querying hardware information for detected disks in a simple way.
/// Originally, this was designed for IDE and it shows.
/// But support for a wide range of interfaces was retrofitted.
///
/// Not all operations are supported by all interface types!
/// Either use [`DiskInfo::interface`] to determine what should be possible, or simply
/// try and handle the [`crate::Status::UNSUPPORTED`] error return value.
///
/// # UEFI Spec Description
/// Provides the basic interfaces to abstract platform information regarding an IDE controller.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(DiskInfoProtocol::GUID)]
pub struct DiskInfo(DiskInfoProtocol);

impl DiskInfo {
    /// Retrieves the interface type of the disk device.
    ///
    /// # Returns
    /// [`DiskInfoInterface`] value representing the disk interface (e.g., IDE, USB, NVME, etc.).
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
    /// # Parameters
    /// - `bfr`: A mutable byte buffer to store the inquiry data.
    ///
    /// # Returns
    /// Length of the response (amount of bytes that were written to the given buffer).
    ///
    /// # Errors
    /// - [`crate::Status::SUCCESS`] The command was accepted without any errors.
    /// - [`crate::Status::NOT_FOUND`] The device does not support this data class.
    /// - [`crate::Status::DEVICE_ERROR`] An error occurred while reading the InquiryData from the device.
    /// - [`crate::Status::BUFFER_TOO_SMALL`] The provided InquiryDataSize buffer is not large enough to store the required data.
    pub fn inquiry(&self, bfr: &mut [u8]) -> crate::Result<usize> {
        let mut len: u32 = bfr.len() as u32;
        unsafe {
            (self.0.inquiry)(&self.0, bfr.as_mut_ptr().cast(), &mut len)
                .to_result_with_val(|| len as usize)
        }
    }

    /// Performs an identify command on the disk device.
    ///
    /// # Parameters
    /// - `bfr`: A mutable byte buffer to store the identification data.
    ///
    /// # Returns
    /// Length of the response (amount of bytes that were written to the given buffer).
    ///
    /// # Errors
    /// - [`crate::Status::SUCCESS`] The command was accepted without any errors.
    /// - [`crate::Status::NOT_FOUND`] The device does not support this data class.
    /// - [`crate::Status::DEVICE_ERROR`] An error occurred while reading the IdentifyData from the device.
    /// - [`crate::Status::BUFFER_TOO_SMALL`] The provided IdentifyDataSize buffer is not large enough to store the required data.
    pub fn identify(&self, bfr: &mut [u8]) -> crate::Result<usize> {
        let mut len: u32 = bfr.len() as u32;
        unsafe {
            (self.0.identify)(&self.0, bfr.as_mut_ptr().cast(), &mut len)
                .to_result_with_val(|| len as usize)
        }
    }

    /// Retrieves sense data from the disk device.
    ///
    /// # Parameters
    /// - `bfr`: A mutable byte buffer to store the sense data.
    ///
    /// # Returns
    /// [`SenseDataInfo`] struct containing the number of bytes of sense data and the number of sense data structures.
    ///
    /// # Errors
    /// - [`crate::Status::SUCCESS`] The command was accepted without any errors.
    /// - [`crate::Status::NOT_FOUND`] The device does not support this data class.
    /// - [`crate::Status::DEVICE_ERROR`] An error occurred while reading the SenseData from the device.
    /// - [`crate::Status::BUFFER_TOO_SMALL`] The provided SenseDataSize buffer is not large enough to store the required data.
    pub fn sense_data(&self, bfr: &mut [u8]) -> crate::Result<SenseDataInfo> {
        let mut len: u32 = bfr.len() as u32;
        let mut number: u8 = 0;
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
    /// This operation provides information about the channel and device identifiers, which can
    /// help determine the device's physical connection point.
    ///
    /// # Returns
    /// [`DeviceLocationInfo`] struct containing the channel and device numbers.
    ///
    /// # Errors
    /// - [`crate::Status::SUCCESS`] The `IdeChannel` and `IdeDevice` values are valid.
    /// - [`crate::Status::UNSUPPORTED`] Not supported by this disk's interface type.
    pub fn bus_location(&self) -> crate::Result<DeviceLocationInfo> {
        let mut ide_channel: u32 = 0; // called ide, but also useful for other interfaces
        let mut ide_device: u32 = 0;
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
