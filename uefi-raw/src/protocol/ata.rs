// SPDX-License-Identifier: MIT OR Apache-2.0

use super::device_path::DevicePathProtocol;
use crate::{Event, Status};
use core::ffi::c_void;
use uguid::{guid, Guid};

bitflags::bitflags! {
    /// ATA Controller attributes.
    ///
    /// These flags defines attributes that describe the nature and capabilities
    /// of the ATA controller represented by this `EFI_ATA_PASS_THRU_PROTOCOL` instance.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(transparent)]
    pub struct AtaPassThruAttributes: u32 {
        /// The protocol interface is for physical devices on the ATA controller.
        ///
        /// This allows access to hardware-level details of devices directly attached to the controller.
        const PHYSICAL = 0x0001;

        /// The protocol interface is for logical devices on the ATA controller.
        ///
        /// Logical devices include RAID volumes and other high-level abstractions.
        const LOGICAL = 0x0002;

        /// The protocol interface supports non-blocking I/O in addition to blocking I/O.
        ///
        /// While all protocol interfaces must support blocking I/O, this attribute indicates
        /// the additional capability for non-blocking operations.
        const NONBLOCKIO = 0x0004;
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct AtaPassThruMode {
    pub attributes: AtaPassThruAttributes,
    pub io_align: u32,
}

newtype_enum! {
    /// Corresponds to the `EFI_ATA_PASS_THRU_PROTOCOL_*` defines.
    #[derive(Default)]
    pub enum AtaPassThruCommandProtocol: u8 => {
        ATA_HARDWARE_RESET = 0x00,
        ATA_SOFTWARE_RESET = 0x01,
        ATA_NON_DATA = 0x02,
        PIO_DATA_IN = 0x04,
        PIO_DATA_OUT = 0x05,
        DMA = 0x06,
        DMA_QUEUED = 0x07,
        DEVICE_DIAGNOSTIC = 0x08,
        DEVICE_RESET = 0x09,
        UDMA_DATA_IN = 0x0A,
        UDMA_DATA_OUT = 0x0B,
        FPDMA = 0x0C,
        RETURN_RESPONSE = 0xFF,
    }
}

newtype_enum! {
    /// Corresponds to the `EFI_ATA_PASS_THRU_LENGTH_*` defines.
    #[derive(Default)]
    pub enum AtaPassThruLength: u8 => {
        BYTES = 0x80,
        MASK = 0x70,
        NO_DATA_TRANSFER = 0x00,
        FEATURES = 0x10,
        SECTOR_COUNT = 0x20,
        TPSIU = 0x30,
        COUNT = 0x0F,
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct AtaStatusBlock {
    pub reserved1: [u8; 2],
    pub status: u8,
    pub error: u8,
    pub sector_number: u8,
    pub cylinder_low: u8,
    pub cylinder_high: u8,
    pub device_head: u8,
    pub sector_number_exp: u8,
    pub cylinder_low_exp: u8,
    pub cylinder_high_exp: u8,
    pub reserved2: u8,
    pub sector_count: u8,
    pub sector_count_exp: u8,
    pub reserved3: [u8; 6],
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct AtaCommandBlock {
    pub reserved1: [u8; 2],
    pub command: u8,
    pub features: u8,
    pub sector_number: u8,
    pub cylinder_low: u8,
    pub cylinder_high: u8,
    pub device_head: u8,
    pub sector_number_exp: u8,
    pub cylinder_low_exp: u8,
    pub cylinder_high_exp: u8,
    pub features_exp: u8,
    pub sector_count: u8,
    pub sector_count_exp: u8,
    pub reserved2: [u8; 6],
}

#[derive(Debug)]
#[repr(C)]
pub struct AtaPassThruCommandPacket {
    pub asb: *mut AtaStatusBlock,
    pub acb: *const AtaCommandBlock,
    pub timeout: u64,
    pub in_data_buffer: *mut c_void,
    pub out_data_buffer: *const c_void,
    pub in_transfer_length: u32,
    pub out_transfer_length: u32,
    pub protocol: AtaPassThruCommandProtocol,
    pub length: AtaPassThruLength,
}

#[derive(Debug)]
#[repr(C)]
pub struct AtaPassThruProtocol {
    pub mode: *const AtaPassThruMode,
    pub pass_thru: unsafe extern "efiapi" fn(
        this: *mut Self,
        port: u16,
        port_multiplier_port: u16,
        packet: *mut AtaPassThruCommandPacket,
        event: Event,
    ) -> Status,
    pub get_next_port: unsafe extern "efiapi" fn(this: *const Self, port: *mut u16) -> Status,
    pub get_next_device: unsafe extern "efiapi" fn(
        this: *const Self,
        port: u16,
        port_multiplier_port: *mut u16,
    ) -> Status,
    pub build_device_path: unsafe extern "efiapi" fn(
        this: *const Self,
        port: u16,
        port_multiplier_port: u16,
        device_path: *mut *const DevicePathProtocol,
    ) -> Status,
    pub get_device: unsafe extern "efiapi" fn(
        this: *const Self,
        device_path: *const DevicePathProtocol,
        port: *mut u16,
        port_multiplier_port: *mut u16,
    ) -> Status,
    pub reset_port: unsafe extern "efiapi" fn(this: *mut Self, port: u16) -> Status,
    pub reset_device:
        unsafe extern "efiapi" fn(this: *mut Self, port: u16, port_multiplier_port: u16) -> Status,
}

impl AtaPassThruProtocol {
    pub const GUID: Guid = guid!("1d3de7f0-0807-424f-aa69-11a54e19a46f");
}
