// ANCHOR: all
// ANCHOR: features
#![no_main]
#![no_std]
// ANCHOR_END: features

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use log::info;

// ANCHOR: use
use uefi::prelude::*;
use uefi::proto::scsi::{
    ExtScsiPassThru, ScsiDeviceLocation, ScsiExtRequestPacket, ScsiIo,
    ScsiRequestPacket,
};

// ANCHOR_END: use

// ANCHOR: entry
#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    // ANCHOR_END: entry
    // ANCHOR: services
    uefi::helpers::init(&mut system_table).unwrap();
    let boot_services = system_table.boot_services();
    // ANCHOR_END: services

    // ANCHOR: params
    /// all api OK, but memory panic when return, maybe the vec.
    test_scsi_io(boot_services);
    // test_ext_scsi_thru(boot_services);
    // ANCHOR_END: params

    // ANCHOR: stall
    boot_services.stall(10_000_000);
    // ANCHOR_END: stall

    // ANCHOR: return
    Status::SUCCESS
}
// ANCHOR_END: return

pub fn test_scsi_io(bt: &BootServices) {
    info!("Running loaded Scsi protocol test");

    let handle = bt
        .get_handle_for_protocol::<ScsiIo>()
        .expect("Failed to get handles for `ScsiIo` protocol");

    let mut scsi_protocol = bt
        .open_protocol_exclusive::<ScsiIo>(handle)
        .expect("Founded ScsiIo Protocol but open failed");

    // value efi_reset_fn is the type of ResetSystemFn, a function pointer

    let result = scsi_protocol.get_device_type();
    info!("SCSI_IO Protocol get device_type: {:?}", result);

    let result = scsi_protocol.io_align();
    info!("SCSI_IO Protocol's io_align: {:?}", result);

    let result = scsi_protocol.get_device_location();
    info!("SCSI_IO Protocol get dev location: {:?}", result);

    let result = scsi_protocol.reset_bus();
    info!("SCSI_IO Protocol reset bus test: {:?}", result);

    let result = scsi_protocol.reset_device();
    info!("SCSI_IO Protocol reset dev test: {:?}", result);

    bt.stall(10_000);
    // let mut data_buffer = vec![0; 40];
    // let scsicmd = commands::TestUnitReady::new();
    // scsicmd.push_to_buffer(&mut data_buffer).expect("TODO: panic message");
    // info!("the data buf = {:?}", data_buffer);

    let mut packet: ScsiRequestPacket = ScsiRequestPacket::default();
    packet.is_a_write_packet = false;
    packet.cdb = vec![0x00, 0, 0, 0, 0, 0x00];
    packet.timeout = 0;
    info!("packet: {:?}", packet);
    let result = scsi_protocol.execute_scsi_command(&mut packet, None);
    info!("=================SCSI_IO Protocol exec scsi command [TestUnitReady] test: {:?}", result);

    let mut packet: ScsiRequestPacket = ScsiRequestPacket::default();
    packet.is_a_write_packet = false;
    packet.cdb = vec![0x12, 0x01, 0x00, 0, 0, 0x00];
    packet.data_buffer = vec![0; 96];
    packet.sense_data = vec![0; 18];
    packet.timeout = 0;
    let result = scsi_protocol.execute_scsi_command(&mut packet, None);
    info!("=================SCSI_IO Protocol exec scsi command [InquiryCommand] test: {:?}", result);

    // now send Req is ok. but seem couldn't receive Resp.
}

/*
// The InquiryCommand


// sense_data with UINT8*18
///
/// Error codes 70h and 71h sense data format
///
typedef struct {
UINT8    Error_Code  : 7;
UINT8    Valid       : 1;
UINT8    Segment_Number;
UINT8    Sense_Key   : 4;
UINT8    Reserved_21 : 1;
UINT8    Ili         : 1;
UINT8    Reserved_22 : 2;
UINT8    Information_3_6[4];
UINT8    Addnl_Sense_Length;        ///< Additional sense length (n-7)
UINT8    Vendor_Specific_8_11[4];
UINT8    Addnl_Sense_Code;            ///< Additional sense code
UINT8    Addnl_Sense_Code_Qualifier;  ///< Additional sense code qualifier
UINT8    Field_Replaceable_Unit_Code; ///< Field replaceable unit code
UINT8    Reserved_15_17[3];
} EFI_SCSI_SENSE_DATA;



// in_data_buf with UINT8 * 96
typedef struct {
  UINT8    Peripheral_Type      : 5;
  UINT8    Peripheral_Qualifier : 3;
  UINT8    DeviceType_Modifier  : 7;
  UINT8    Rmb                  : 1;
  UINT8    Version;
  UINT8    Response_Data_Format;
  UINT8    Addnl_Length;
  UINT8    Reserved_5_95[95 - 5 + 1];
} EFI_SCSI_INQUIRY_DATA;
*/

pub fn test_ext_scsi_thru(bt: &BootServices) {
    info!("Running loaded Ext Scsi Thru protocol test");

    let handle = bt
        .get_handle_for_protocol::<ExtScsiPassThru>()
        .expect("Failed to get handles for `ExtScsiThru` protocol");

    let mut escsi_protocol = bt
        .open_protocol_exclusive::<ExtScsiPassThru>(handle)
        .expect("Founded ExtScsiThru Protocol but open failed");

    // value efi_reset_fn is the type of ResetSystemFn, a function pointer

    let result = escsi_protocol.mode();
    info!("EXT_SCSI_THRU Protocol's mode: {:?}", result);

    let mut targets: Vec<u8> = vec![0xFF; 10];
    let target = ScsiDeviceLocation::new(&mut targets[0], 0);
    let result = escsi_protocol.build_device_path(target);
    info!("EXT_SCSI_THRU Protocol build_device_path: {:?}", result);

    // let device_path = result.expect("couldn't got the dev path");
    // // DevicePath has DevicePathFromText ffi
    // let result = escsi_protocol.get_target_lun(&*device_path);
    // info!("EXT_SCSI_THRU Protocol build_device_path: {:?}", result);

    // let location = result.unwrap().clone();

    let location = target;

    let result = escsi_protocol.get_next_target();
    info!("EXT_SCSI_THRU Protocol get_next_target: {:?}", result);

    // let result = escsi_protocol.reset_target_lun(location.clone());
    // info!("EXT_SCSI_THRU Protocol reset dev test: {:?}", result);

    let result = escsi_protocol.reset_channel();
    info!("EXT_SCSI_THRU Protocol reset bus test: {:?}", result);

    bt.stall(10_000);

    let mut ext_packet = ScsiExtRequestPacket::default();
    ext_packet.is_a_write_packet = false;
    ext_packet.cdb = vec![0x00, 0, 0, 0, 0, 0x00];
    ext_packet.timeout = 0;
    let result = escsi_protocol.pass_thru(location.clone(), ext_packet, None);
    info!("=================EXT_SCSI_THRU Protocol exec scsi command [TestUnitReady] test: {:?}", result);

    let mut ext_packet = ScsiExtRequestPacket::default();
    ext_packet.is_a_write_packet = false;
    ext_packet.cdb = vec![0x12, 0x01, 0x00, 0, 0, 0x00];
    ext_packet.data_buffer = vec![0; 96];
    ext_packet.sense_data = vec![0; 18];
    ext_packet.timeout = 0;
    let result = escsi_protocol.pass_thru(location.clone(), ext_packet, None);
    info!("=================EXT_SCSI_THRU Protocol exec scsi command [InquiryCommand] test: {:?}", result);

    // now send Req is ok. but seem couldn't receive Resp.
}
