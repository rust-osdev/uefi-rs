// SPDX-License-Identifier: MIT OR Apache-2.0

//! USB I/O protocols.
//!
//! These protocols can be used to interact with and configure USB devices.

pub mod io;

pub use uefi_raw::protocol::usb::{
    AsyncUsbTransferCallback, ConfigDescriptor, DeviceDescriptor, EndpointDescriptor,
    InterfaceDescriptor, UsbTransferStatus,
};
