//! EFI SCSI I/O protocols.

use alloc::vec::Vec;
use core::ffi::c_void;
use core::ptr::null_mut;

use uefi_raw::protocol::device_path::DevicePathProtocol;
use uefi_raw::protocol::scsi;
use uefi_raw::protocol::scsi::{
    DataDirection, ExtDataDirection, ExtHostAdapterStatus, ExtScsiIoScsiRequestPacket,
    ExtScsiPassThruMode, ExtScsiPassThruProtocol, ExtTargetStatus, HostAdapterStatus,
    ScsiIoProtocol, ScsiIoScsiRequestPacket, TargetStatus,
};

use crate::{Event, Result, StatusExt};
use crate::proto::device_path::DevicePath;
use crate::proto::unsafe_protocol;

/// Protocol for who running in the EFI boot services environment such as code, typically drivers, able to access SCSI devices.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(ScsiIoProtocol::GUID)]
pub struct ScsiIo(ScsiIoProtocol);

/// Represents a scsi device location which {target, lun}.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ScsiDeviceLocation {
    /// Target ID
    pub target: *mut u8,
    /// Logical Unit Number
    pub lun: u64,
}

impl Default for ScsiDeviceLocation {
    fn default() -> Self {
        ScsiDeviceLocation {
            target: null_mut(),
            lun: 0,
        }
    }
}
impl ScsiIo {
    /// Retrieves the device type information of the SCSI Device.
    pub fn get_device_type(&self) -> Result<scsi::DeviceType> {
        let mut device_type = scsi::DeviceType::default();
        unsafe { (self.0.get_device_type)(&self.0, &mut device_type) }
            .to_result_with_val(|| device_type)
    }

    /// Retrieves the SCSI device location in the SCSI channel.
    pub fn get_device_location(&self) -> Result<ScsiDeviceLocation> {
        let mut location = ScsiDeviceLocation::default();
        unsafe { (self.0.get_device_location)(&self.0, &mut location.target, &mut location.lun) }
            .to_result_with_val(|| location)
    }
    /// Resets the SCSI Bus that the SCSI Device is attached to.
    pub fn reset_bus(&mut self) -> Result {
        unsafe { (self.0.reset_bus)(&mut self.0) }.to_result()
    }
    /// Resets the SCSI Device that is specified by the device handle that the SCSI I/O Protocol is attached.
    pub fn reset_device(&mut self) -> Result {
        unsafe { (self.0.reset_device)(&mut self.0) }.to_result()
    }

    /// Sends a SCSI Request Packet to the SCSI Device for execution.
    ///TODO:  ScsiIoScsiRequestPacket must to refactor
    pub fn execute_scsi_command(&self, packet: &mut ScsiRequestPacket, event: Event) -> Result {
        unsafe {
            (self.0.execute_scsi_command)(
                &self.0,
                &mut (packet.convert_to_request_packet()),
                event.as_ptr(),
            )
        }
            .to_result()
    }

    /// the value of ioAlign
    pub fn io_align(&self) -> Result<u32> {
        Ok(self.0.io_align)
    }
}

/// Extended SCSI Pass Thru Protocol.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(ExtScsiPassThruProtocol::GUID)]
pub struct ExtScsiPassThru(ExtScsiPassThruProtocol);

impl ExtScsiPassThru {
    /// the value of mode which is type ExtScsiPassThruMode.
    pub fn mode(&self) -> Result<ExtScsiPassThruMode> {
        Ok(self.0.mode)
    }
    /// Sends a SCSI Request Packet to a SCSI device that is attached to the SCSI channel.
    pub fn pass_thru(
        &mut self,
        location: ScsiDeviceLocation,
        mut packet: ScsiExtRequestPacket,
        event: Event,
    ) -> Result {
        unsafe {
            (self.0.pass_thru)(
                &mut self.0,
                location.target,
                location.lun,
                &mut (packet.convert_to_request_packet()),
                event.as_ptr(),
            )
        }
        .to_result()
    }

    /// Used to translate a device path node to a Target ID and LUN.
    pub fn get_next_target_lun(&mut self) -> Result<ScsiDeviceLocation> {
        let mut location = ScsiDeviceLocation::default();
        unsafe { (self.0.get_next_target_lun)(&self.0, &mut location.target, &mut location.lun) }
            .to_result_with_val(|| location)
    }

    /// Used to allocate and build a device path node for a SCSI device on a SCSI channel.
    pub fn build_device_path(&mut self, location: ScsiDeviceLocation) -> Result<&DevicePath> {
        let mut path = &mut DevicePathProtocol {
            major_type: 0,
            sub_type: 0,
            length: [0, 0],
        } as *mut DevicePathProtocol;
        unsafe {
            let status =
                (self.0.build_device_path)(&mut self.0, location.target, location.lun, &mut path);
            if status.is_success() {
                Ok(DevicePath::from_ffi_ptr(path.cast()))
            } else {
                Err(status.into())
            }
        }
    }

    /// Used to translate a device path node to a Target ID and LUN.
    pub fn get_target_lun(&mut self, device_path: &DevicePath) -> Result<ScsiDeviceLocation> {
        let device_path_ptr: *const uefi_raw::protocol::device_path::DevicePathProtocol =
            device_path.as_ffi_ptr().cast();

        let mut location = ScsiDeviceLocation::default();
        unsafe {
            (self.0.get_target_lun)(
                &self.0,
                device_path_ptr,
                &mut location.target,
                &mut location.lun,
            )
        }
        .to_result_with_val(|| location)
    }

    /// Resets a SCSI channel. This operation resets all the SCSI devices connected to the SCSI channel.
    pub fn reset_channel(&mut self) -> Result {
        unsafe { (self.0.reset_channel)(&mut self.0) }.to_result()
    }

    /// Resets a SCSI logical unit that is connected to a SCSI channel.
    pub fn reset_target_lun(&mut self, location: ScsiDeviceLocation) -> Result {
        unsafe { (self.0.reset_target_lun)(&mut self.0, location.target, location.lun) }.to_result()
    }

    /// Used to retrieve the list of legal Target IDs for SCSI devices on a SCSI channel.
    pub fn get_next_target(&mut self) -> Result<ScsiDeviceLocation> {
        let mut location = ScsiDeviceLocation::default();
        unsafe { (self.0.get_next_target)(&self.0, &mut location.target) }
            .to_result_with_val(|| location)
    }
}

/// the rust FFI for `EFI_EXT_SCSI_PASS_THRU_SCSI_REQUEST_PACKET`
#[derive(Debug, Default, Clone)]
pub struct ScsiExtRequestPacket {
    /// whether the request is written scsi
    pub is_a_write_packet: bool,
    /// timeout
    pub timeout: u64,
    /// data_buffer is `in_data_buffer` or `out_data_buffer`
    pub data_buffer: Vec<u8>,
    /// SCSI's cdb, refer to T10 SPC
    pub cdb: Vec<u8>,
    /// SCSI's sense data, refer to T10 SPC
    pub sense_data: Vec<u8>,
    /// uefi_raw::protocol::scsi::ExtDataDirection
    pub data_direction: ExtDataDirection,
    /// uefi_raw::protocol::scsi::ExtHostAdapterStatus
    pub host_adapter_status: ExtHostAdapterStatus,
    /// uefi_raw::protocol::scsi::ExtTargetStatus
    pub target_status: ExtTargetStatus,
}

impl ScsiExtRequestPacket {
    /// convert FFI `ScsiExtRequestPacket` to raw UEFI SCSI request packet `EFI_EXT_SCSI_PASS_THRU_SCSI_REQUEST_PACKET`
    pub fn convert_to_request_packet(&mut self) -> ExtScsiIoScsiRequestPacket {
        if self.is_a_write_packet {
            self._convert_to_write_request_packet()
        } else {
            self._convert_to_read_request_packet()
        }
    }
    fn _convert_to_read_request_packet(&mut self) -> ExtScsiIoScsiRequestPacket {
        let packet: ExtScsiIoScsiRequestPacket = ExtScsiIoScsiRequestPacket {
            timeout: self.timeout,

            in_data_buffer: self.data_buffer.as_mut_ptr() as *mut c_void,
            out_data_buffer: null_mut(),
            sense_data: self.sense_data.as_mut_ptr() as *mut c_void,
            cdb: self.cdb.as_mut_ptr() as *mut c_void,

            in_transfer_length: self.data_buffer.len() as u32,
            out_transfer_length: 0,
            sense_data_length: self.sense_data.len() as u8,
            cdb_length: self.cdb.len() as u8,

            data_direction: self.data_direction,
            host_adapter_status: self.host_adapter_status,
            target_status: self.target_status,
        };
        packet
    }

    fn _convert_to_write_request_packet(&mut self) -> ExtScsiIoScsiRequestPacket {
        let packet: ExtScsiIoScsiRequestPacket = ExtScsiIoScsiRequestPacket {
            timeout: self.timeout,

            in_data_buffer: null_mut(),
            out_data_buffer: self.data_buffer.as_mut_ptr() as *mut c_void,
            sense_data: self.sense_data.as_mut_ptr() as *mut c_void,
            cdb: self.cdb.as_mut_ptr() as *mut c_void,

            in_transfer_length: 0,
            out_transfer_length: self.data_buffer.len() as u32,
            sense_data_length: self.sense_data.len() as u8,
            cdb_length: self.cdb.len() as u8,

            data_direction: self.data_direction,
            host_adapter_status: self.host_adapter_status,
            target_status: self.target_status,
        };
        packet
    }
}

/// the rust FFI for `EFI_SCSI_IO_SCSI_REQUEST_PACKET`
#[derive(Debug, Default, Clone)]
pub struct ScsiRequestPacket {
    /// whether the request is written scsi
    is_a_write_packet: bool,
    /// timeout
    pub timeout: u64,
    /// data_buffer is `in_data_buffer` or `out_data_buffer`
    pub data_buffer: Vec<u8>,
    /// SCSI's cdb, refer to T10 SPC
    pub cdb: Vec<u8>,
    /// SCSI's sense data, refer to T10 SPC
    pub sense_data: Vec<u8>,
    /// uefi_raw::protocol::scsi::DataDirection
    pub data_direction: DataDirection,
    /// uefi_raw::protocol::scsi::HostAdapterStatus
    pub host_adapter_status: HostAdapterStatus,
    /// uefi_raw::protocol::scsi::TargetStatus
    pub target_status: TargetStatus,
}

impl ScsiRequestPacket {
    /// convert FFI `ScsiRequestPacket` to raw UEFI SCSI request packet `EFI_SCSI_IO_SCSI_REQUEST_PACKET`
    pub fn convert_to_request_packet(&mut self) -> ScsiIoScsiRequestPacket {
        if self.is_a_write_packet {
            self._convert_to_write_request_packet()
        } else {
            self._convert_to_read_request_packet()
        }
    }
    fn _convert_to_read_request_packet(&mut self) -> ScsiIoScsiRequestPacket {
        let packet: ScsiIoScsiRequestPacket = ScsiIoScsiRequestPacket {
            timeout: self.timeout,

            in_data_buffer: self.data_buffer.as_mut_ptr() as *mut c_void,
            out_data_buffer: null_mut(),
            sense_data: self.sense_data.as_mut_ptr() as *mut c_void,
            cdb: self.cdb.as_mut_ptr() as *mut c_void,

            in_transfer_length: self.data_buffer.len() as u32,
            out_transfer_length: 0,
            sense_data_length: self.sense_data.len() as u8,
            cdb_length: self.cdb.len() as u8,

            data_direction: self.data_direction,
            host_adapter_status: self.host_adapter_status,
            target_status: self.target_status,
        };
        packet
    }

    fn _convert_to_write_request_packet(&mut self) -> ScsiIoScsiRequestPacket {
        let packet: ScsiIoScsiRequestPacket = ScsiIoScsiRequestPacket {
            timeout: self.timeout,

            in_data_buffer: null_mut(),
            out_data_buffer: self.data_buffer.as_mut_ptr() as *mut c_void,
            sense_data: self.sense_data.as_mut_ptr() as *mut c_void,
            cdb: self.cdb.as_mut_ptr() as *mut c_void,

            in_transfer_length: 0,
            out_transfer_length: self.data_buffer.len() as u32,
            sense_data_length: self.sense_data.len() as u8,
            cdb_length: self.cdb.len() as u8,

            data_direction: self.data_direction,
            host_adapter_status: self.host_adapter_status,
            target_status: self.target_status,
        };
        packet
    }
}
