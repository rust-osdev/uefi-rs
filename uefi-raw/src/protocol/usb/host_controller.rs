// SPDX-License-Identifier: MIT OR Apache-2.0

use core::ffi;

use bitflags::bitflags;

use crate::{guid, Boolean, Guid, Status};

use super::{AsyncUsbTransferCallback, DataDirection, DeviceRequest, UsbTransferStatus};

newtype_enum! {
    pub enum Speed: u8 => {
        FULL = 0,
        LOW = 1,
        HIGH = 2,
        SUPER = 3,
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ResetAttributes: u16 {
        /// Send a global reset signal to the USB bus.
        const RESET_GLOBAL = 0x0001;
        /// Reset the USB host controller hardware.
        ///
        /// No reset signal will be sent to the USB bus.
        const RESET_HOST = 0x0002;
        /// Send a global reset signal to the USB bus.
        ///
        /// Even if a debug port has been enabled, this still resets the host controller.
        const RESET_GLOBAL_WITH_DEBUG = 0x0004;
        /// Reset the USB host controller hardware.
        ///
        /// Even if a debug port has been enabled, this still resets the host controller.
        const RESET_HOST_WITH_DEBUG = 0x0008;
    }
}

newtype_enum! {
    pub enum HostControllerState: i32 => {
        HALT = 0,
        OPERATIONAL = 1,
        SUSPEND = 2,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct TransactionTranslator {
    pub hub_address: u8,
    pub port_number: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct UsbPortStatus {
    pub port_status: PortStatus,
    pub port_change_status: PortChangeStatus,
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct PortStatus: u16 {
        const CONNECTION = 0x0001;
        const ENABLE = 0x0002;
        const SUSPEND = 0x0004;
        const OVER_CURRENT = 0x0008;
        const RESET = 0x0010;
        const POWER = 0x0100;
        const LOW_SPEED = 0x0200;
        const HIGH_SPEED = 0x0400;
        const SUPER_SPEED = 0x0800;
        const OWNER = 0x2000;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct PortChangeStatus: u16 {
        const CONNECTION = 0x0001;
        const ENABLE = 0x0002;
        const SUSPEND = 0x0004;
        const OVER_CURRENT = 0x0008;
        const RESET = 0x0010;
    }
}

newtype_enum! {
    pub enum PortFeature: i32 => {
        ENABLE = 1,
        SUSPEND = 2,
        RESET = 4,
        POWER = 8,
        OWNER = 13,
        CONNECT_CHANGE = 16,
        ENABLE_CHANGE = 17,
        SUSPEND_CHANGE = 18,
        OVER_CURRENT_CHARGE = 19,
        RESET_CHANGE = 20,
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Usb2HostControllerProtocol {
    pub get_capability: unsafe extern "efiapi" fn(
        this: *const Self,
        max_speed: *mut Speed,
        port_number: *mut u8,
        is_64_bit_capable: *mut u8,
    ) -> Status,
    pub reset: unsafe extern "efiapi" fn(this: *mut Self, attributes: ResetAttributes) -> Status,
    pub get_state:
        unsafe extern "efiapi" fn(this: *mut Self, state: *mut HostControllerState) -> Status,
    pub set_state: unsafe extern "efiapi" fn(this: *mut Self, state: HostControllerState) -> Status,
    pub control_transfer: unsafe extern "efiapi" fn(
        this: *mut Self,
        device_address: u8,
        device_speed: Speed,
        maximum_packet_length: usize,
        request: *const DeviceRequest,
        transfer_direction: DataDirection,
        data: *mut ffi::c_void,
        data_length: *mut usize,
        timeout: usize,
        translator: *const TransactionTranslator,
        transfer_result: *mut UsbTransferStatus,
    ) -> Status,
    pub bulk_transfer: unsafe extern "efiapi" fn(
        this: *mut Self,
        device_address: u8,
        endpoint_address: u8,
        device_speed: Speed,
        maximum_packet_length: usize,
        data_buffers_number: u8,
        data: *const *const ffi::c_void,
        data_length: *mut usize,
        data_toggle: *mut u8,
        timeout: usize,
        translator: *const TransactionTranslator,
        transfer_result: *mut UsbTransferStatus,
    ) -> Status,
    pub async_interrupt_transfer: unsafe extern "efiapi" fn(
        this: *mut Self,
        device_address: u8,
        endpoint_address: u8,
        device_speed: Speed,
        maximum_packet_length: usize,
        is_new_transfer: Boolean,
        data_toggle: *mut u8,
        polling_interval: usize,
        data_length: usize,
        translator: *const TransactionTranslator,
        callback_function: AsyncUsbTransferCallback,
        context: *mut ffi::c_void,
    ) -> Status,
    pub sync_interrupt_transfer: unsafe extern "efiapi" fn(
        this: *mut Self,
        device_address: u8,
        endpoint_address: u8,
        device_speed: Speed,
        maximum_packet_length: usize,
        data: *mut ffi::c_void,
        data_length: *mut usize,
        data_toggle: *mut u8,
        timeout: usize,
        translator: *const TransactionTranslator,
        transfer_result: *mut UsbTransferStatus,
    ) -> Status,
    pub isochronous_transfer: unsafe extern "efiapi" fn(
        this: *mut Self,
        device_address: u8,
        endpoint_address: u8,
        device_speed: Speed,
        maximum_packet_length: usize,
        data_buffers_number: u8,
        data: *const *const ffi::c_void,
        data_length: usize,
        translator: *const TransactionTranslator,
        transfer_result: *mut UsbTransferStatus,
    ) -> Status,
    pub async_isochronous_transfer: unsafe extern "efiapi" fn(
        this: *mut Self,
        device_address: u8,
        endpoint_address: u8,
        device_speed: Speed,
        maximum_packet_length: usize,
        data_buffers_number: u8,
        data: *const *const ffi::c_void,
        data_length: usize,
        translator: *const TransactionTranslator,
        isochronous_callback: AsyncUsbTransferCallback,
        context: *mut ffi::c_void,
    ) -> Status,
    pub get_root_hub_port_status: unsafe extern "efiapi" fn(
        this: *mut Self,
        port_number: u8,
        port_status: *mut UsbPortStatus,
    ) -> Status,
    pub set_root_hub_port_feature: unsafe extern "efiapi" fn(
        this: *mut Self,
        port_number: u8,
        port_feature: PortFeature,
    ) -> Status,
    pub clear_root_hub_port_feature:
        unsafe extern "efiapi" fn(this: *mut Self, port_number: u8, feature: PortFeature) -> Status,

    pub major_revision: u16,
    pub minor_revision: u16,
}

impl Usb2HostControllerProtocol {
    pub const GUID: Guid = guid!("3e745226-9818-45b6-a2ac-d7cd0e8ba2bc");
}
