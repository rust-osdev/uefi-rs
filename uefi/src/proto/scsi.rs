//! EFI SCSI I/O protocols.

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::ptr;
use core::ptr::null_mut;

use log::info;

use uefi_raw::protocol::scsi;
use uefi_raw::protocol::scsi::{
    DataDirection, ExtDataDirection, ExtHostAdapterStatus, ExtScsiIoScsiRequestPacket,
    ExtScsiPassThruMode, ExtScsiPassThruProtocol, ExtTargetStatus, HostAdapterStatus,
    ScsiIoProtocol, ScsiIoScsiRequestPacket, TargetStatus,
};

use crate::proto::device_path::build::{acpi, hardware, messaging, DevicePathBuilder};
use crate::proto::device_path::DevicePath;
use crate::proto::unsafe_protocol;
use crate::{Event, Result, StatusExt};

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

/// Extended SCSI Pass Thru Protocol.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(ExtScsiPassThruProtocol::GUID)]
pub struct ExtScsiPassThru(ExtScsiPassThruProtocol);

impl ExtScsiPassThru {
    /// the value of mode which is type ExtScsiPassThruMode.
    pub fn mode(&self) -> Result<ExtScsiPassThruMode> {
        unsafe { Ok(*self.0.mode) }
    }
    /// Sends a SCSI Request Packet to a SCSI device that is attached to the SCSI channel.
    pub fn pass_thru(
        &mut self,
        location: ScsiDeviceLocation,
        mut packet: ScsiExtRequestPacket,
        event: Option<Event>,
    ) -> Result {
        let raw_packet = &mut (packet.convert_auto_request_packet());
        info!("raw_packet = {:?}", raw_packet);
        let event_arg = match event {
            Some(event) => event.as_ptr(),
            None => ptr::null_mut(),
        };

        unsafe {
            (self.0.pass_thru)(
                &mut self.0,
                location.target,
                location.lun,
                raw_packet,
                event_arg,
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
    pub fn build_device_path(&mut self, location: ScsiDeviceLocation) -> Result<Box<DevicePath>> {
        let mut v = Vec::new();
        let path = DevicePathBuilder::with_vec(&mut v)
            .push(&acpi::Acpi {
                hid: 0x41d0_0a03,
                uid: 0x0000_0000,
            })
            .unwrap()
            .push(&hardware::Pci {
                function: 0x00,
                device: 0x19,
            })
            .unwrap()
            .push(&messaging::Scsi {
                target_id: 0,
                logical_unit_number: 0,
            })
            .unwrap()
            .finalize()
            .expect("failed to build dev path");
        let mut device_path_ptr: *mut uefi_raw::protocol::device_path::DevicePathProtocol =
            unsafe { *path.as_ffi_ptr().cast() };
        // let path_ptr = &mut path?;
        unsafe {
            (self.0.build_device_path)(
                &mut self.0,
                location.target,
                location.lun,
                &mut device_path_ptr,
            )
        }
        .to_result_with_val(|| path.to_boxed())
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
        let mut id = 0;
        let mut location = ScsiDeviceLocation::new(&mut id, 0);
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
    /// auto convert FFI `ScsiExtRequestPacket` to raw UEFI SCSI request packet `EFI_EXT_SCSI_PASS_THRU_SCSI_REQUEST_PACKET`
    pub fn convert_zero_request_packet(&mut self) -> ExtScsiIoScsiRequestPacket {
        let packet: ExtScsiIoScsiRequestPacket = ExtScsiIoScsiRequestPacket {
            timeout: self.timeout,

            in_data_buffer: null_mut(),
            out_data_buffer: null_mut(),
            sense_data: self.sense_data.as_mut_ptr() as *mut c_void,
            cdb: self.cdb.as_mut_ptr() as *mut c_void,

            in_transfer_length: 0,
            out_transfer_length: 0,
            sense_data_length: self.sense_data.len() as u8,
            cdb_length: self.cdb.len() as u8,

            data_direction: self.data_direction,
            host_adapter_status: self.host_adapter_status,
            target_status: self.target_status,
        };
        packet
    }

    /// convert auto
    pub fn convert_auto_request_packet(&mut self) -> ExtScsiIoScsiRequestPacket {
        ExtScsiIoScsiRequestPacket {
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
