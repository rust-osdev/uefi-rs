//! EFI SCSI I/O protocols.

use alloc::vec::Vec;
use core::ffi::c_void;
use core::ptr;
use core::ptr::null_mut;

use log::info;

use uefi_raw::protocol::scsi;
use uefi_raw::protocol::scsi::{
    DataDirection, HostAdapterStatus, ScsiIoProtocol, ScsiIoScsiRequestPacket, TargetStatus,
};

use crate::{Event, Result, StatusExt};
use crate::proto::unsafe_protocol;

/// Protocol for who running in the EFI boot services environment such as code, typically drivers, able to access SCSI devices.
/// see example at `uefi-test-runner/examples/scsi.rs`
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

impl ScsiDeviceLocation {
    /// constructor for ScsiDeviceLocation {target, lun}
    pub fn new(target: *mut u8, lun: u64) -> Self {
        ScsiDeviceLocation { target, lun }
    }
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
    pub fn execute_scsi_command(
        &self,
        packet: &mut ScsiRequestPacket,
        event: Option<Event>,
    ) -> Result<ScsiRequestPacket> {
        info!("before: ffi_packet = {:?}", packet);
        let in_packet = &mut (packet.convert_auto_request_packet());
        info!("before: raw_packet = {:?}", in_packet);

        let event_arg = match event {
            Some(event) => event.as_ptr(),
            None => ptr::null_mut(),
        };

        let status = unsafe { (self.0.execute_scsi_command)(&self.0, in_packet, event_arg) };
        info!("after: raw_packet = {:?}", in_packet);
        // TODO: print log with raw dat/len about `ScsiIoScsiRequestPacket`

        status.to_result_with_val(|| packet.sync_from_request_packet(in_packet))
    }

    /// the value of ioAlign
    pub fn io_align(&self) -> Result<u32> {
        Ok(self.0.io_align)
    }
}

/// the rust FFI for `EFI_SCSI_IO_SCSI_REQUEST_PACKET`
#[derive(Debug, Default, Clone)]
pub struct ScsiRequestPacket {
    /// whether the request is written scsi
    pub is_a_write_packet: bool,
    /// timeout
    pub timeout: u64,
    /// data_buffer is `in_data_buffer` or `out_data_buffer`
    pub data_buffer: Vec<u8>,
    /// SCSI's cdb, refer to T10 SPC
    pub cdb: Vec<u8>,
    /// SCSI's sense data, refer to T10 SPC, scsi resp return it
    pub sense_data: Vec<u8>,
    /// uefi_raw::protocol::scsi::DataDirection
    pub data_direction: DataDirection,
    /// uefi_raw::protocol::scsi::HostAdapterStatus, scsi resp status
    pub host_adapter_status: HostAdapterStatus,
    /// uefi_raw::protocol::scsi::TargetStatus, scsi resp status
    pub target_status: TargetStatus,
}

impl ScsiRequestPacket {
    /// convert FFI `ScsiRequestPacket` to raw UEFI SCSI request packet `EFI_SCSI_IO_SCSI_REQUEST_PACKET`
    pub fn convert_zero_request_packet(&mut self) -> ScsiIoScsiRequestPacket {
        let packet: ScsiIoScsiRequestPacket = ScsiIoScsiRequestPacket {
            timeout: self.timeout,

            in_data_buffer: null_mut(),
            out_data_buffer: null_mut(),
            sense_data: null_mut(),
            cdb: self.cdb.as_mut_ptr() as *mut c_void,

            in_transfer_length: 0,
            out_transfer_length: 0,
            sense_data_length: 0,
            cdb_length: self.cdb.len() as u8,

            data_direction: self.data_direction,
            host_adapter_status: self.host_adapter_status,
            target_status: self.target_status,
        };
        packet
    }
    // auto convert
    /// convert auto
    pub fn convert_auto_request_packet(&mut self) -> ScsiIoScsiRequestPacket {
        ScsiIoScsiRequestPacket {
            timeout: 0,

            in_data_buffer: if (self.data_buffer.len() != 0) && !self.is_a_write_packet {
                self.data_buffer.as_mut_ptr().cast::<c_void>()
            } else {
                null_mut()
            },
            out_data_buffer: if self.data_buffer.len() != 0 && self.is_a_write_packet {
                self.data_buffer.as_mut_ptr().cast::<c_void>()
            } else {
                null_mut()
            },

            sense_data: if self.sense_data.len() != 0 {
                self.sense_data.as_mut_ptr().cast::<c_void>()
            } else {
                null_mut()
            },
            cdb: if self.cdb.len() != 0 {
                self.cdb.as_mut_ptr().cast::<c_void>()
            } else {
                null_mut()
            },

            in_transfer_length: if !self.is_a_write_packet {
                self.data_buffer.len() as u32
            } else {
                0
            },
            out_transfer_length: if self.is_a_write_packet {
                self.data_buffer.len() as u32
            } else {
                0
            },
            cdb_length: self.cdb.len() as u8,
            sense_data_length: self.sense_data.len() as u8,

            data_direction: Default::default(),
            host_adapter_status: Default::default(),
            target_status: Default::default(),
        }
    }

    /// convert FFI `ScsiRequestPacket` to raw UEFI SCSI request packet `EFI_SCSI_IO_SCSI_REQUEST_PACKET`
    pub fn convert_to_request_packet(&mut self) -> ScsiIoScsiRequestPacket {
        if self.is_a_write_packet {
            self._convert_to_write_request_packet()
        } else {
            self._convert_to_read_request_packet()
        }
    }

    /// `ScsiRequestPacket` FFI sync from raw_packet `ScsiIoScsiRequestPacket` by ptr.
    pub fn sync_from_request_packet(
        &mut self,
        raw_packet: &mut ScsiIoScsiRequestPacket,
    ) -> ScsiRequestPacket {
        unsafe {
            self.timeout = raw_packet.timeout;
            // c (void* data, int len) => rust Vec<u8>

            self.cdb = Vec::from_raw_parts(
                raw_packet.cdb as *mut u8,
                raw_packet.cdb_length as usize,
                isize::MAX as usize,
            );
            self.sense_data = Vec::from_raw_parts(
                raw_packet.sense_data as *mut u8,
                raw_packet.sense_data_length as usize,
                isize::MAX as usize,
            );
            self.data_buffer = if self.is_a_write_packet {
                Vec::from_raw_parts(
                    raw_packet.in_data_buffer as *mut u8,
                    raw_packet.in_transfer_length as usize,
                    isize::MAX as usize,
                )
            } else {
                Vec::from_raw_parts(
                    raw_packet.out_data_buffer as *mut u8,
                    raw_packet.out_transfer_length as usize,
                    isize::MAX as usize,
                )
            };

            self.data_direction = raw_packet.data_direction;
            self.host_adapter_status = raw_packet.host_adapter_status;
            self.target_status = raw_packet.target_status;
        }
        self.clone()
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
