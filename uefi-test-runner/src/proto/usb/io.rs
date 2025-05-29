// SPDX-License-Identifier: MIT OR Apache-2.0

use core::mem;
use uefi::proto::usb::DeviceDescriptor;
use uefi::proto::usb::io::{ControlTransfer, UsbIo};
use uefi::{Status, boot};

const DEVICE_TO_HOST: u8 = 1 << 7;
const STANDARD_REQUEST: u8 = 0b00 << 5;
const DEVICE_RECIPIENT: u8 = 0b0_0000;
const GET_DESCRIPTOR_REQUEST: u8 = 6;
const DEVICE_DESCRIPTOR: u8 = 1;

/// This test iterates through all of the exposed active USB interfaces and
/// performs checks on each to validate that descriptor acquisition and control
/// transfers work correctly.
pub fn test() {
    info!("Testing USB I/O protocol");

    let handles = boot::locate_handle_buffer(boot::SearchType::from_proto::<UsbIo>())
        .expect("failed to acquire USB I/O handles");

    for handle in handles.iter().copied() {
        let mut io = boot::open_protocol_exclusive::<UsbIo>(handle)
            .expect("failed to open USB I/O protocol");

        let device = io
            .device_descriptor()
            .expect("failed to acquire USB device descriptor");
        io.config_descriptor()
            .expect("failed to acquire USB config descriptor");
        io.interface_descriptor()
            .expect("failed to acquire USB interface descriptor");

        for endpoint_index in 0..16 {
            let result = io.endpoint_descriptor(endpoint_index);
            if result
                .as_ref()
                .is_err_and(|error| error.status() == Status::NOT_FOUND)
            {
                continue;
            }

            result.expect("failed to acquire USB endpoint descriptor");
        }

        let supported_languages = io
            .supported_languages()
            .expect("failed to acquire supported language list");
        let test_language = supported_languages[0];

        for string_index in 0..=u8::MAX {
            let result = io.string_descriptor(test_language, string_index);
            if result
                .as_ref()
                .is_err_and(|error| error.status() == Status::NOT_FOUND)
            {
                continue;
            }

            result.expect("failed to acquire string descriptor");
        }

        let mut buffer = [0u8; mem::size_of::<DeviceDescriptor>()];

        io.control_transfer(
            DEVICE_TO_HOST | STANDARD_REQUEST | DEVICE_RECIPIENT,
            GET_DESCRIPTOR_REQUEST,
            u16::from(DEVICE_DESCRIPTOR) << 8,
            0,
            ControlTransfer::DataIn(&mut buffer[..mem::size_of::<DeviceDescriptor>()]),
            0,
        )
        .expect("failed control transfer");
        unsafe {
            assert_eq!(
                device,
                buffer.as_ptr().cast::<DeviceDescriptor>().read_unaligned()
            )
        }
    }
}
