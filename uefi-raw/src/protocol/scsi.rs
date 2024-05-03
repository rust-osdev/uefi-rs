use core::ffi::c_void;

use bitflags::bitflags;

use crate::{Event, guid, Guid, Status};
use crate::protocol::device_path::DevicePathProtocol;

bitflags! {
    /// DataDirection
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct DataDirection: u8 {
        const EFI_SCSI_IO_DATA_DIRECTION_READ            = 0;
        const EFI_SCSI_IO_DATA_DIRECTION_WRITE           = 1;
        const EFI_SCSI_IO_DATA_DIRECTION_BIDIRECTIONAL   = 2;
    }
}

bitflags! {
    /// HostAdapterStatus
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct HostAdapterStatus: u8 {
        const EFI_SCSI_IO_STATUS_HOST_ADAPTER_OK            =        0x00  ;
        const EFI_SCSI_IO_STATUS_HOST_ADAPTER_TIMEOUT_COMMAND       =0x09          ;
        const EFI_SCSI_IO_STATUS_HOST_ADAPTER_TIMEOUT               =0x0b          ;
        const EFI_SCSI_IO_STATUS_HOST_ADAPTER_MESSAGE_REJECT        =0x0d          ;
        const EFI_SCSI_IO_STATUS_HOST_ADAPTER_BUS_RESET             =0x0e          ;
        const EFI_SCSI_IO_STATUS_HOST_ADAPTER_PARITY_ERROR          =0x0f          ;
        const EFI_SCSI_IO_STATUS_HOST_ADAPTER_REQUEST_SENSE_FAILED  =0x10          ;
        const EFI_SCSI_IO_STATUS_HOST_ADAPTER_SELECTION_TIMEOUT     =0x11          ;
        const EFI_SCSI_IO_STATUS_HOST_ADAPTER_DATA_OVERRUN_UNDERRUN =0x12          ;
        const EFI_SCSI_IO_STATUS_HOST_ADAPTER_BUS_FREE              =0x13          ;
        const EFI_SCSI_IO_STATUS_HOST_ADAPTER_PHASE_ERROR           =0x14          ;
        const EFI_SCSI_IO_STATUS_HOST_ADAPTER_OTHER                 =0x7f          ;

    }
}

bitflags! {
    /// TargetStatus
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct TargetStatus: u8 {
        const EFI_SCSI_IO_STATUS_TARGET_GOOD                         =0x00;
        const EFI_SCSI_IO_STATUS_TARGET_CHECK_CONDITION              =0x02;
        const EFI_SCSI_IO_STATUS_TARGET_CONDITION_MET                =0x04;
        const EFI_SCSI_IO_STATUS_TARGET_BUSY                         =0x08;
        const EFI_SCSI_IO_STATUS_TARGET_INTERMEDIATE                 =0x10;
        const EFI_SCSI_IO_STATUS_TARGET_INTERMEDIATE_CONDITION_METn  =0x14;
        const EFI_SCSI_IO_STATUS_TARGET_RESERVATION_CONFLICT         =0x18;
        const EFI_SCSI_IO_STATUS_TARGET_COMMAND_TERMINATED           =0x22;
        const EFI_SCSI_IO_STATUS_TARGET_QUEUE_FULL                   =0x28;
    }
}

#[derive(Debug)]
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
    pub data_direction: DataDirection,
    pub host_adapter_status: HostAdapterStatus,
    pub target_status: TargetStatus,
    pub sense_data_length: u8,
}

bitflags! {
    /// DeviceType
    /// Defined in the SCSI Primary Commands standard (e.g., SPC-4)
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct DeviceType: u8  {
        const DISK               = 0x00; // Disk device
        const TAPE              = 0x01; // Tape device
        const PRINTER           = 0x02;// Printer
        const PROCESSOR         = 0x03;// Processor
        const WORM              = 0x04;// Write-once read-multiple
        const CDROM             = 0x05;// CD or DVD device
        const SCANNER           = 0x06;// Scanner device
        const OPTICAL           = 0x07;// Optical memory device
        const MEDIUMCHANGER     = 0x08;// Medium Changer device
        const COMMUNICATION     = 0x09;// Communications device


        const MFI_A               =   0x0A; // Obsolete
        const MFI_B               =   0x0B; // Obsolete
        const MFI_RAID            =   0x0C; // Storage array controller

        const MFI_SES             =   0x0D; // Enclosure services device
        const MFI_RBC             =   0x0E; // Simplified direct-access


        const MFI_OCRW            =   0x0F; // Optical card reader/

        const MFI_BRIDGE          =   0x10; // Bridge Controller

        const MFI_OSD             =   0x11; // Object-based Storage

        const RESERVED_LOW    =   0x12; // Reserved (low)
        const RESERVED_HIGH   =   0x1E; // Reserved (high)
        const UNKNOWN         =   0x1F; // Unknown no device type
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct ScsiIoProtocol {
    //TODO: return deviceType
    pub get_device_type:
        unsafe extern "efiapi" fn(this: *const Self, device_type: *mut DeviceType) -> Status,
    //TODO: raw pointer need to fixed, see uefi-rs service code like pointer *u8
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

/// 15.4. EFI SCSI I/O Protocol
impl ScsiIoProtocol {
    pub const GUID: Guid = guid!("932f47e6-2362-4002-803e-3cd54b138f85");
}

// TODO: see device_path.rs, then see 15.5. SCSI Device Paths and 15.6. SCSI Pass Thru Device Paths

#[derive(Debug)]
#[repr(C)]
pub struct ExtScsiPassThruProtocol {
    pub mode: ExtScsiPassThruMode,
    pub pass_thru: unsafe extern "efiapi" fn(
        this: *const Self,
        target: *mut u8,
        lun: u64,
        packet: *mut ExtScsiIoScsiRequestPacket,
        event: Event,
    ) -> Status,
    pub get_next_target_lun:
        unsafe extern "efiapi" fn(this: *const Self, target: *mut *mut u8, lun: *mut u64) -> Status,
    pub build_device_path: unsafe extern "efiapi" fn(
        this: *mut Self,
        target: *mut u8,
        lun: u64,
        device_path: *mut *mut DevicePathProtocol,
    ) -> Status,
    pub get_target_lun: unsafe extern "efiapi" fn(
        this: *const Self,
        device_path: *const DevicePathProtocol,
        target: *mut *mut u8,
        lun: *mut u64,
    ) -> Status,

    pub reset_channel: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
    pub reset_target_lun:
        unsafe extern "efiapi" fn(this: *mut Self, target: *mut u8, lun: u64) -> Status,
    pub get_next_target:
        unsafe extern "efiapi" fn(this: *const Self, target: *mut *mut u8) -> Status,
}
/// 15.7. Extended SCSI Pass Thru Protocol
impl ExtScsiPassThruProtocol {
    pub const GUID: Guid = guid!("143b7632-b81b-4cb7-abd3-b625a5b9bffe");
}

bitflags! {
    /// Attributes
    /// TODO: #define TARGET_MAX_BYTES 0x10
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Attributes: u32 {
        const EFI_EXT_SCSI_PASS_THRU_ATTRIBUTES_PHYSICAL     = 0x0001;
        const EFI_EXT_SCSI_PASS_THRU_ATTRIBUTES_LOGICAL      = 0x0002;
        const EFI_EXT_SCSI_PASS_THRU_ATTRIBUTES_NONBLOCKIO   = 0x0004;
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct ExtScsiPassThruMode {
    pub adapter_id: u32,
    pub attributes: Attributes,
    pub io_align: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct ExtScsiIoScsiRequestPacket {
    pub timeout: u64,

    pub in_data_buffer: *mut c_void,
    pub out_data_buffer: *mut c_void,
    pub sense_data: *mut c_void,
    pub cdb: *mut c_void,

    pub in_transfer_length: u32,
    pub out_transfer_length: u32,

    pub cdb_length: u8,
    pub data_direction: ExtDataDirection,
    pub host_adapter_status: ExtHostAdapterStatus,
    pub target_status: ExtTargetStatus,
    pub sense_data_length: u8,
}

bitflags! {
    /// Ext DataDirection
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ExtDataDirection: u8 {
        const EFI_EXT_SCSI_DATA_DIRECTION_READ            = 0;
        const EFI_EXT_SCSI_DATA_DIRECTION_WRITE           = 1;
        const EFI_EXT_SCSI_DATA_DIRECTION_BIDIRECTIONAL   = 2;
    }
}

bitflags! {
    /// Ext HostAdapterStatus
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ExtHostAdapterStatus: u8 {
        const EFI_EXT_SCSI_STATUS_HOST_ADAPTER_OK                    =0x00;
        const EFI_EXT_SCSI_STATUS_HOST_ADAPTER_TIMEOUT_COMMAND       =0x09;
        const EFI_EXT_SCSI_STATUS_HOST_ADAPTER_TIMEOUT               =0x0b;
        const EFI_EXT_SCSI_STATUS_HOST_ADAPTER_MESSAGE_REJECT        =0x0d;
        const EFI_EXT_SCSI_STATUS_HOST_ADAPTER_BUS_RESET             =0x0e;
        const EFI_EXT_SCSI_STATUS_HOST_ADAPTER_PARITY_ERROR          =0x0f;
        const EFI_EXT_SCSI_STATUS_HOST_ADAPTER_REQUEST_SENSE_FAILED  =0x10;
        const EFI_EXT_SCSI_STATUS_HOST_ADAPTER_SELECTION_TIMEOUT     =0x11;
        const EFI_EXT_SCSI_STATUS_HOST_ADAPTER_DATA_OVERRUN_UNDERRUN =0x12;
        const EFI_EXT_SCSI_STATUS_HOST_ADAPTER_BUS_FREE              =0x13;
        const EFI_EXT_SCSI_STATUS_HOST_ADAPTER_PHASE_ERROR           =0x14;
        const EFI_EXT_SCSI_STATUS_HOST_ADAPTER_OTHER                 =0x7f;
    }
}

bitflags! {
    /// ExtTargetStatus
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ExtTargetStatus: u8 {
        const EFI_EXT_SCSI_STATUS_TARGET_GOOD                        = 0x00;
        const EFI_EXT_SCSI_STATUS_TARGET_CHECK_CONDITION             = 0x02;
        const EFI_EXT_SCSI_STATUS_TARGET_CONDITION_MET               = 0x04;
        const EFI_EXT_SCSI_STATUS_TARGET_BUSY                        = 0x08;
        const EFI_EXT_SCSI_STATUS_TARGET_INTERMEDIATE                = 0x10;
        const EFI_EXT_SCSI_STATUS_TARGET_INTERMEDIATE_CONDITION_MET  = 0x14;
        const EFI_EXT_SCSI_STATUS_TARGET_RESERVATION_CONFLICT        = 0x18;
        const EFI_EXT_SCSI_STATUS_TARGET_TASK_SET_FULL               = 0x28;
        const EFI_EXT_SCSI_STATUS_TARGET_ACA_ACTIVE                  = 0x30;
        const EFI_EXT_SCSI_STATUS_TARGET_TASK_ABORTED                = 0x40;
    }
}
