// SPDX-License-Identifier: MIT OR Apache-2.0

use super::device_path::DevicePathProtocol;
use crate::{guid, Event, Guid, Status};
use core::ffi::c_void;

pub const SCSI_TARGET_MAX_BYTES: usize = 0x10;

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
    /// The timeout, in 100 ns units, for the execution of this SCSI Request Packet.
    ///
    /// A `timeout` value of 0 indicates that the function will wait indefinitely for
    /// the execution to complete. If the execution time exceeds the specified `timeout`
    /// (greater than 0), the function will return `EFI_TIMEOUT`.
    pub timeout: u64,

    /// A pointer to the data buffer for reading from the device in read and bidirectional commands.
    ///
    /// - For write and non-data commands where `in_transfer_length` is 0, this field is optional and may be `NULL`.
    /// - If not `NULL`, the buffer must meet the alignment requirement specified by the `IoAlign` field
    ///   in the `EFI_EXT_SCSI_PASS_THRU_MODE` structure.
    pub in_data_buffer: *mut c_void,

    /// A pointer to the data buffer for writing to the device in write and bidirectional commands.
    ///
    /// - For read and non-data commands where `out_transfer_length` is 0, this field is optional and may be `NULL`.
    /// - If not `NULL`, the buffer must meet the alignment requirement specified by the `IoAlign` field
    ///   in the `EFI_EXT_SCSI_PASS_THRU_MODE` structure.
    pub out_data_buffer: *mut c_void,

    /// A pointer to the sense data generated during execution of the SCSI Request Packet.
    ///
    /// - If `sense_data_length` is 0, this field is optional and may be `NULL`.
    /// - It is recommended to allocate a buffer of at least 252 bytes to ensure the entire sense data can be captured.
    /// - If not `NULL`, the buffer must meet the alignment requirement specified by the `IoAlign` field
    ///   in the `EFI_EXT_SCSI_PASS_THRU_MODE` structure.
    pub sense_data: *mut c_void,

    /// A pointer to the Command Data Block (CDB) buffer to be sent to the SCSI device.
    ///
    /// The CDB contains the SCSI command to be executed by the device.
    pub cdb: *mut c_void,

    /// The input size (in bytes) of the `in_data_buffer`, and the number of bytes transferred on output.
    ///
    /// - On input: Specifies the size of `in_data_buffer`.
    /// - On output: Specifies the number of bytes successfully transferred.
    /// - If the size exceeds the controller's capability, no data is transferred, the field is updated
    ///   with the number of transferable bytes, and `EFI_BAD_BUFFER_SIZE` is returned.
    pub in_transfer_length: u32,

    /// The input size (in bytes) of the `out_data_buffer`, and the number of bytes transferred on output.
    ///
    /// - On input: Specifies the size of `out_data_buffer`.
    /// - On output: Specifies the number of bytes successfully transferred.
    /// - If the size exceeds the controller's capability, no data is transferred, the field is updated
    ///   with the number of transferable bytes, and `EFI_BAD_BUFFER_SIZE` is returned.
    pub out_transfer_length: u32,

    /// The length (in bytes) of the Command Data Block (CDB).
    ///
    /// Standard values for CDB length are typically 6, 10, 12, or 16 bytes. Other values are possible
    /// for variable-length CDBs.
    pub cdb_length: u8,

    /// The direction of data transfer for the SCSI Request Packet.
    pub data_direction: ScsiIoDataDirection,

    /// The status of the host adapter when the SCSI Request Packet was executed.
    pub host_adapter_status: ScsiIoHostAdapterStatus,

    /// The status of the target device when the SCSI Request Packet was executed.
    pub target_status: ScsiIoTargetStatus,

    /// The size (in bytes) of the `sense_data` buffer on input, and the number of bytes written on output.
    ///
    /// - On input: Specifies the size of the `sense_data` buffer.
    /// - On output: Specifies the number of bytes written to the buffer.
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

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ExtScsiPassThruMode {
    pub adapter_id: u32,
    pub attributes: u32,
    pub io_align: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct ExtScsiPassThruProtocol {
    pub passthru_mode: *const ExtScsiPassThruMode,
    pub pass_thru: unsafe extern "efiapi" fn(
        this: *mut Self,
        target: *const u8,
        lun: u64,
        packet: *mut ScsiIoScsiRequestPacket,
        event: Event,
    ) -> Status,
    pub get_next_target_lun:
        unsafe extern "efiapi" fn(this: *const Self, target: *mut *mut u8, lun: *mut u64) -> Status,
    pub build_device_path: unsafe extern "efiapi" fn(
        this: *const Self,
        target: *const u8,
        lun: u64,
        device_path: *mut *const DevicePathProtocol,
    ) -> Status,
    pub get_target_lun: unsafe extern "efiapi" fn(
        this: *const Self,
        device_path: *const DevicePathProtocol,
        target: *mut *const u8,
        lun: *mut u64,
    ) -> Status,
    pub reset_channel: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
    pub reset_target_lun:
        unsafe extern "efiapi" fn(this: *mut Self, target: *const u8, lun: u64) -> Status,
    pub get_next_target:
        unsafe extern "efiapi" fn(this: *const Self, target: *mut *mut u8) -> Status,
}

impl ExtScsiPassThruProtocol {
    pub const GUID: Guid = guid!("143b7632-b81b-4cb7-abd3-b625a5b9bffe");
}
