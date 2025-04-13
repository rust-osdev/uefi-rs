// SPDX-License-Identifier: MIT OR Apache-2.0

//! Protocols for converting between UEFI strings and [`DevicePath`]/[`DevicePathNode`].

// Note on return types: the specification of the conversion functions
// is a little unusual in that they return a pointer rather than
// `EFI_STATUS`. A NULL pointer is used to indicate an error, and the
// spec says that will only happen if the input pointer is null (which
// can't happen here since we use references as input, not pointers), or
// if there is insufficient memory. So we treat any NULL output as an
// `OUT_OF_RESOURCES` error.

use crate::data_types::PoolString;
use crate::mem::PoolAllocation;
use crate::proto::device_path::{DevicePath, DevicePathNode};
use crate::proto::unsafe_protocol;
use crate::{CStr16, Result, Status};
use core::ptr::NonNull;
use uefi_raw::protocol::device_path::{DevicePathFromTextProtocol, DevicePathToTextProtocol};

use super::{PoolDevicePath, PoolDevicePathNode};

/// Parameter for [`DevicePathToText`] that alters the output format.
///
/// * `DisplayOnly(false)` produces parseable output.
/// * `DisplayOnly(true)` produces output that _may_ be shorter and not
///   parseable.
///
/// Example of how a node's text representation may be altered by this
/// parameter:
/// * `DisplayOnly(false)`: `Ata(Primary,Master,0x1)`
/// * `DisplayOnly(true)`: `Ata(0x1)`
#[derive(Clone, Copy, Debug)]
pub struct DisplayOnly(pub bool);

/// Parameter for [`DevicePathToText`] that alters the output format.
///
/// * `AllowShortcuts(false)`: node names are based only on the node's type and
///   subtype.
/// * `AllowShortcuts(true)` _may_ alter the node name based on other fields
///   within the node.
///
/// Example of how a node's text representation may be altered by this
/// parameter:
/// * `AllowShortcuts(false)`: `VenMsg(E0C14753-F9BE-11D2-9A0C-0090273FC14D)`
/// * `AllowShortcuts(true)`: `VenPcAnsi()`
#[derive(Clone, Copy, Debug)]
pub struct AllowShortcuts(pub bool);

/// Protocol for converting a [`DevicePath`] or `DevicePathNode`] to a string.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(DevicePathToTextProtocol::GUID)]
pub struct DevicePathToText(DevicePathToTextProtocol);

impl DevicePathToText {
    /// Convert a [`DevicePathNode`] to a [`PoolString`].
    ///
    /// Returns an [`OUT_OF_RESOURCES`] error if there is insufficient
    /// memory for the conversion.
    ///
    /// [`OUT_OF_RESOURCES`]: Status::OUT_OF_RESOURCES
    pub fn convert_device_node_to_text(
        &self,
        device_node: &DevicePathNode,
        display_only: DisplayOnly,
        allow_shortcuts: AllowShortcuts,
    ) -> Result<PoolString> {
        let text = unsafe {
            (self.0.convert_device_node_to_text)(
                device_node.as_ffi_ptr().cast(),
                display_only.0.into(),
                allow_shortcuts.0.into(),
            )
        };
        unsafe { PoolString::new(text.cast()) }
    }

    /// Convert a [`DevicePath`] to a [`PoolString`].
    ///
    /// Returns an [`OUT_OF_RESOURCES`] error if there is insufficient
    /// memory for the conversion.
    ///
    /// [`OUT_OF_RESOURCES`]: Status::OUT_OF_RESOURCES
    pub fn convert_device_path_to_text(
        &self,
        device_path: &DevicePath,
        display_only: DisplayOnly,
        allow_shortcuts: AllowShortcuts,
    ) -> Result<PoolString> {
        let text = unsafe {
            (self.0.convert_device_path_to_text)(
                device_path.as_ffi_ptr().cast(),
                display_only.0.into(),
                allow_shortcuts.0.into(),
            )
        };
        unsafe { PoolString::new(text.cast()) }
    }
}

/// Protocol for converting a string to a [`DevicePath`] or `DevicePathNode`].
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol("05c99a21-c70f-4ad2-8a5f-35df3343f51e")]
pub struct DevicePathFromText(DevicePathFromTextProtocol);

impl DevicePathFromText {
    /// Convert a [`CStr16`] to a [`DevicePathNode`].
    ///
    /// If a non-device-node character is encountered, the rest of the string is ignored.
    ///
    /// Returns an [`OUT_OF_RESOURCES`] error if there is insufficient
    /// memory for the conversion.
    ///
    /// [`OUT_OF_RESOURCES`]: Status::OUT_OF_RESOURCES
    pub fn convert_text_to_device_node(
        &self,
        text_device_node: &CStr16,
    ) -> Result<PoolDevicePathNode> {
        unsafe {
            let ptr = (self.0.convert_text_to_device_node)(text_device_node.as_ptr().cast());
            NonNull::new(ptr.cast_mut())
                .map(|p| PoolDevicePathNode(PoolAllocation::new(p.cast())))
                .ok_or(Status::OUT_OF_RESOURCES.into())
        }
    }

    /// Convert a [`CStr16`] to a [`DevicePath`].
    ///
    /// If a non-device-node character is encountered, the rest of the string is ignored.
    ///
    /// Returns an [`OUT_OF_RESOURCES`] error if there is insufficient
    /// memory for the conversion.
    ///
    /// [`OUT_OF_RESOURCES`]: Status::OUT_OF_RESOURCES
    pub fn convert_text_to_device_path(&self, text_device_path: &CStr16) -> Result<PoolDevicePath> {
        unsafe {
            let ptr = (self.0.convert_text_to_device_path)(text_device_path.as_ptr().cast());
            NonNull::new(ptr.cast_mut())
                .map(|p| PoolDevicePath(PoolAllocation::new(p.cast())))
                .ok_or(Status::OUT_OF_RESOURCES.into())
        }
    }
}
