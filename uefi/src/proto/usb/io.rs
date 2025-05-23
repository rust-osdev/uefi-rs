// SPDX-License-Identifier: MIT OR Apache-2.0

//! USB I/O protocol.

use core::ffi;

use uefi_macros::unsafe_protocol;
use uefi_raw::protocol::usb::io::UsbIoProtocol;
use uefi_raw::protocol::usb::{
    ConfigDescriptor, DataDirection, DeviceDescriptor, DeviceRequest, EndpointDescriptor,
    InterfaceDescriptor, UsbTransferStatus,
};

use crate::data_types::PoolString;
use crate::{Char16, Result, StatusExt};

/// USB I/O protocol.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(UsbIoProtocol::GUID)]
pub struct UsbIo(UsbIoProtocol);

impl UsbIo {
    /// Performs a USB Control transfer, allowing the driver to communicate with the USB device.
    pub fn control_transfer(
        &mut self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        transfer: ControlTransfer,
        timeout: u32,
    ) -> Result<(), UsbTransferStatus> {
        let (direction, buffer_ptr, length) = match transfer {
            ControlTransfer::None => (DataDirection::NO_DATA, core::ptr::null_mut(), 0),
            ControlTransfer::DataIn(buffer) => (
                DataDirection::DATA_IN,
                buffer.as_ptr().cast_mut(),
                buffer.len(),
            ),
            ControlTransfer::DataOut(buffer) => (
                DataDirection::DATA_OUT,
                buffer.as_ptr().cast_mut(),
                buffer.len(),
            ),
        };

        let request_type = if direction == DataDirection::DATA_IN {
            request_type | 0x80
        } else if direction == DataDirection::DATA_OUT {
            request_type & !0x80
        } else {
            request_type
        };

        let mut device_request = DeviceRequest {
            request_type,
            request,
            value,
            index,
            length: length as u16,
        };
        let mut status = UsbTransferStatus::default();

        unsafe {
            (self.0.control_transfer)(
                &mut self.0,
                &mut device_request,
                direction,
                timeout,
                buffer_ptr.cast::<ffi::c_void>(),
                length,
                &mut status,
            )
        }
        .to_result_with_err(|_| status)
    }

    /// Sends the provided buffer to a USB device over a bulk transfer pipe.
    ///
    /// Returns the number of bytes that were actually sent to the device.
    pub fn sync_bulk_send(
        &mut self,
        endpoint: u8,
        buffer: &[u8],
        timeout: usize,
    ) -> Result<usize, UsbTransferStatus> {
        let mut status = UsbTransferStatus::default();
        let mut length = buffer.len();

        unsafe {
            (self.0.bulk_transfer)(
                &mut self.0,
                endpoint & !0x80,
                buffer.as_ptr().cast_mut().cast::<ffi::c_void>(),
                &mut length,
                timeout,
                &mut status,
            )
        }
        .to_result_with_err(|_| status)
        .map(|()| length)
    }

    /// Fills the provided buffer with data from a USB device over a bulk transfer pipe.
    ///
    /// Returns the number of bytes that were actually received from the device.
    pub fn sync_bulk_receive(
        &mut self,
        endpoint: u8,
        buffer: &mut [u8],
        timeout: usize,
    ) -> Result<usize, UsbTransferStatus> {
        let mut status = UsbTransferStatus::default();
        let mut length = buffer.len();

        unsafe {
            (self.0.bulk_transfer)(
                &mut self.0,
                endpoint | 0x80,
                buffer.as_ptr().cast_mut().cast::<ffi::c_void>(),
                &mut length,
                timeout,
                &mut status,
            )
        }
        .to_result_with_err(|_| status)
        .map(|()| length)
    }

    /// Sends the provided buffer to a USB device through a synchronous interrupt transfer.
    pub fn sync_interrupt_send(
        &mut self,
        endpoint: u8,
        buffer: &[u8],
        timeout: usize,
    ) -> Result<usize, UsbTransferStatus> {
        let mut status = UsbTransferStatus::default();
        let mut length = buffer.len();

        unsafe {
            (self.0.sync_interrupt_transfer)(
                &mut self.0,
                endpoint & !0x80,
                buffer.as_ptr().cast_mut().cast::<ffi::c_void>(),
                &mut length,
                timeout,
                &mut status,
            )
        }
        .to_result_with_err(|_| status)
        .map(|()| length)
    }

    /// Fills the provided buffer with data from a USB device through a synchronous interrupt
    /// transfer.
    pub fn sync_interrupt_receive(
        &mut self,
        endpoint: u8,
        buffer: &mut [u8],
        timeout: usize,
    ) -> Result<usize, UsbTransferStatus> {
        let mut status = UsbTransferStatus::default();
        let mut length = buffer.len();

        unsafe {
            (self.0.sync_interrupt_transfer)(
                &mut self.0,
                endpoint | 0x80,
                buffer.as_ptr().cast_mut().cast::<ffi::c_void>(),
                &mut length,
                timeout,
                &mut status,
            )
        }
        .to_result_with_err(|_| status)
        .map(|()| length)
    }

    /// Sends the provided buffer to a USB device over an isochronous transfer pipe.
    pub fn sync_isochronous_send(
        &mut self,
        endpoint: u8,
        buffer: &[u8],
    ) -> Result<(), UsbTransferStatus> {
        let mut status = UsbTransferStatus::default();

        unsafe {
            (self.0.isochronous_transfer)(
                &mut self.0,
                endpoint & !0x80,
                buffer.as_ptr().cast_mut().cast::<ffi::c_void>(),
                buffer.len(),
                &mut status,
            )
        }
        .to_result_with_err(|_| status)
    }

    /// Fills the provided buffer with data from a USB device over an isochronous transfer pipe.
    pub fn sync_isochronous_receive(
        &mut self,
        endpoint: u8,
        buffer: &mut [u8],
    ) -> Result<(), UsbTransferStatus> {
        let mut status = UsbTransferStatus::default();

        unsafe {
            (self.0.isochronous_transfer)(
                &mut self.0,
                endpoint | 0x80,
                buffer.as_mut_ptr().cast::<ffi::c_void>(),
                buffer.len(),
                &mut status,
            )
        }
        .to_result_with_err(|_| status)
    }

    /// Returns information about USB devices, including the device's class, subclass, and number
    /// of configurations.
    pub fn device_descriptor(&mut self) -> Result<DeviceDescriptor> {
        let mut device_descriptor = unsafe { core::mem::zeroed() };

        unsafe { (self.0.get_device_descriptor)(&mut self.0, &mut device_descriptor) }
            .to_result_with_val(|| device_descriptor)
    }

    /// Returns information about the active configuration of the USB device.
    pub fn config_descriptor(&mut self) -> Result<ConfigDescriptor> {
        let mut config_descriptor = unsafe { core::mem::zeroed() };

        unsafe { (self.0.get_config_descriptor)(&mut self.0, &mut config_descriptor) }
            .to_result_with_val(|| config_descriptor)
    }

    /// Returns information about the interface of the USB device.
    pub fn interface_descriptor(&mut self) -> Result<InterfaceDescriptor> {
        let mut interface_descriptor = unsafe { core::mem::zeroed() };

        unsafe { (self.0.get_interface_descriptor)(&mut self.0, &mut interface_descriptor) }
            .to_result_with_val(|| interface_descriptor)
    }

    /// Returns information about the interface of the USB device.
    pub fn endpoint_descriptor(&mut self, endpoint: u8) -> Result<EndpointDescriptor> {
        let mut endpoint_descriptor = unsafe { core::mem::zeroed() };

        unsafe { (self.0.get_endpoint_descriptor)(&mut self.0, endpoint, &mut endpoint_descriptor) }
            .to_result_with_val(|| endpoint_descriptor)
    }

    /// Returns the string associated with `string_id` in the language associated with `lang_id`.
    pub fn string_descriptor(&mut self, lang_id: u16, string_id: u8) -> Result<PoolString> {
        let mut string_ptr = core::ptr::null_mut();

        unsafe { (self.0.get_string_descriptor)(&mut self.0, lang_id, string_id, &mut string_ptr) }
            .to_result()?;
        unsafe { PoolString::new(string_ptr.cast::<Char16>()) }
    }

    /// Returns all of the language ID codes that the USB device supports.
    pub fn supported_languages(&mut self) -> Result<&[u16]> {
        let mut lang_id_table_ptr = core::ptr::null_mut();
        let mut lang_id_table_size = 0;

        unsafe {
            (self.0.get_supported_languages)(
                &mut self.0,
                &mut lang_id_table_ptr,
                &mut lang_id_table_size,
            )
        }
        .to_result_with_val(|| unsafe {
            core::slice::from_raw_parts(lang_id_table_ptr, usize::from(lang_id_table_size))
        })
    }

    /// Resets and reconfigures the USB controller.
    ///
    /// This function should work for all USB devices except USB Hub Controllers.
    pub fn port_reset(&mut self) -> Result {
        unsafe { (self.0.port_reset)(&mut self.0) }.to_result()
    }
}

/// Controls what type of USB control transfer operation should occur.
#[derive(Debug)]
pub enum ControlTransfer<'buffer> {
    /// The USB control transfer has no data phase.
    None,
    /// The USB control transfer has an input data phase.
    DataIn(&'buffer mut [u8]),
    /// The USB control transfer has an output data phase.
    DataOut(&'buffer [u8]),
}
