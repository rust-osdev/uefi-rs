// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{guid, Event, Guid, Status};
use core::ffi::c_void;

newtype_enum! {
    /// Corresponds to the `EFI_SCSI_IO_TYPE_*` defines.
    #[derive(Default)]
    pub enum ScsiIoType: u8  => {
        DISK = 0x00,
        TAPE = 0x01,
        PRINTER = 0x02,
        PROCESSOR = 0x03,
        WRITE_ONCE_READ_MULTIPLE = 0x04,
        CDROM = 0x05,
        SCANNER = 0x06,
        OPTICAL = 0x07,
        MEDIUM_CHANGER = 0x08,
        COMMUNICATION = 0x09,
        RAID = 0x0c,
        ENCLOSURE_SERVICES = 0x0d,
        REDUCED_BLOCK_COMMANDS = 0x0e,
        OPTICAL_CARD_READER_WRITER = 0x0f,
        BRIDGE_CONTROLLER = 0x10,
        OBJECT_BASED_STORAGE = 0x11,
        RESERVED_LOW = 0x12,
        RESERVED_HIGH = 0x1e,
        UNKNOWN = 0x1f,
    }
}

newtype_enum! {
    /// Corresponds to the `EFI_SCSI_IO_DATA_DIRECTION_*` defines.
    #[derive(Default)]
    pub enum ScsiIoDataDirection: u8 => {
        READ = 0,
        WRITE = 1,
        BIDIRECTIONAL = 2,
    }
}

newtype_enum! {
    /// Corresponds to the `EFI_SCSI_IO_STATUS_HOST_ADAPTER_*` defines.
    #[derive(Default)]
    pub enum ScsiIoHostAdapterStatus: u8 => {
       OK = 0x00,
       TIMEOUT_COMMAND = 0x09,
       TIMEOUT = 0x0b,
       MESSAGE_REJECT = 0x0d,
       BUS_RESET = 0x0e,
       PARITY_ERROR = 0x0f,
       REQUEST_SENSE_FAILED = 0x10,
       SELECTION_TIMEOUT = 0x11,
       DATA_OVERRUN_UNDERRUN = 0x12,
       BUS_FREE = 0x13,
       PHASE_ERROR = 0x14,
       OTHER = 0x7f,
    }
}

newtype_enum! {
    /// Corresponds to the `EFI_SCSI_IO_STATUS_TARGET_*` defines.
    #[derive(Default)]
    pub enum ScsiIoTargetStatus: u8 => {
        GOOD = 0x00,
        CHECK_CONDITION = 0x02,
        CONDITION_MET = 0x04,
        BUSY = 0x08,
        INTERMEDIATE = 0x10,
        INTERMEDIATE_CONDITION_MET = 0x14,
        RESERVATION_CONFLICT = 0x18,
        COMMAND_TERMINATED = 0x22,
        QUEUE_FULL = 0x28,
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ScsiIoScsiRequestPacket {
    pub timeout: u64,
    pub in_data_buffer: *mut c_void,
    pub out_data_buffer: *mut c_void,
    pub sense_data: *mut c_void,
    pub cdb: *mut c_void,
    pub in_transfer_length: u32,
    pub out_transfer_length: u32,
    pub cdb_length: u8,
    pub data_direction: ScsiIoDataDirection,
    pub host_adapter_status: ScsiIoHostAdapterStatus,
    pub target_status: ScsiIoTargetStatus,
    pub sense_data_length: u8,
}

#[derive(Debug)]
#[repr(C)]
pub struct ScsiIoProtocol {
    pub get_device_type:
        unsafe extern "efiapi" fn(this: *const Self, device_type: *mut ScsiIoType) -> Status,
    pub get_device_location:
        unsafe extern "efiapi" fn(this: *const Self, target: *mut *mut u8, lun: *mut u64) -> Status,
    pub reset_bus: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
    pub reset_device: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
    pub execute_scsi_command: unsafe extern "efiapi" fn(
        this: *const Self,
        packet: *mut ScsiIoScsiRequestPacket,
        event: Event,
    ) -> Status,
    pub io_align: u32,
}

impl ScsiIoProtocol {
    pub const GUID: Guid = guid!("932f47e6-2362-4002-803e-3cd54b138f85");
}
