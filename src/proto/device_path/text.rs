//! `DevicePathToText` and `DevicePathFromText` Protocol

use crate::{
    proto::{device_path::DevicePath, Protocol},
    unsafe_guid, CStr16, Char16,
};

/// Device Path to Text protocol.
///
/// This protocol provides common utility functions for converting device
/// nodes and device paths to a text representation.
#[repr(C)]
#[unsafe_guid("8b843e20-8132-4852-90cc-551a4e4a7f1c")]
#[derive(Protocol)]
pub struct DevicePathToText {
    convert_device_node_to_text: unsafe extern "efiapi" fn(
        device_node: *const DevicePath,
        display_only: bool,
        allow_shortcuts: bool,
    ) -> *const Char16,
    convert_device_path_to_text: unsafe extern "efiapi" fn(
        device_path: *const DevicePath,
        display_only: bool,
        allow_shortcuts: bool,
    ) -> *const Char16,
}

impl DevicePathToText {
    /// Convert a device node to its text representation.
    ///
    /// The `display_only` parameter controls whether the longer (parseable) or
    /// shorter (display-only) form of the conversion is used. If `display_only`
    /// is TRUE, then the shorter text representation of the display node is
    /// used, where applicable. If `display_only` is FALSE, then the longer text
    /// representation of the display node is used.
    ///
    /// The `allow_shortcuts` is FALSE, then the shortcut forms of text
    /// representation for a device node cannot be used. A shortcut form
    /// is one which uses information other than the type or subtype. If
    /// `allow_shortcuts is TRUE, then the shortcut forms of text
    /// representation for a device node can be used, where applicable.
    ///
    /// Returns `None` if `device_node` was NULL or there was insufficient memory.
    pub fn convert_device_node_to_text(
        &self,
        device_node: &DevicePath,
        display_only: bool,
        allow_shortcuts: bool,
    ) -> Option<&CStr16> {
        let text_device_node = unsafe {
            (self.convert_device_node_to_text)(device_node, display_only, allow_shortcuts)
        };
        unsafe { Some(CStr16::from_ptr(text_device_node.as_ref()?)) }
    }

    /// Convert a device path to its text representation.
    ///
    /// The `display_only` parameter controls whether the longer (parseable) or
    /// shorter (display-only) form of the conversion is used. If `display_only`
    /// is TRUE, then the shorter text representation of the display node is
    /// used, where applicable. If `display_only` is FALSE, then the longer text
    /// representation of the display node is used.
    ///
    /// The `allow_shortcuts` is FALSE, then the shortcut forms of text
    /// representation for a device node cannot be used. A shortcut form
    /// is one which uses information other than the type or subtype. If
    /// `allow_shortcuts is TRUE, then the shortcut forms of text
    /// representation for a device node can be used, where applicable.
    ///
    /// Returns `None` if `device_path` was NULL or there was insufficient memory.
    pub fn convert_device_path_to_text(
        &self,
        device_path: &DevicePath,
        display_only: bool,
        allow_shortcuts: bool,
    ) -> Option<&CStr16> {
        let text_device_path = unsafe {
            (self.convert_device_path_to_text)(device_path, display_only, allow_shortcuts)
        };
        unsafe { Some(CStr16::from_ptr(text_device_path.as_ref()?)) }
    }
}

/// Device Path from Text protocol.
///
/// This protocol provides common utilities for converting text to
/// device paths and device nodes.
#[repr(C)]
#[unsafe_guid("05c99a21-c70f-4ad2-8a5f-35df3343f51e")]
#[derive(Protocol)]
pub struct DevicePathFromText {
    convert_text_to_device_node:
        unsafe extern "efiapi" fn(text_device_node: *const Char16) -> *const DevicePath,
    convert_text_to_device_path:
        unsafe extern "efiapi" fn(text_device_path: *const Char16) -> *const DevicePath,
}

impl DevicePathFromText {
    /// Convert text to the binary representation of a device node.
    ///
    /// `text_device_node` is the text representation of a device node.
    /// Conversion starts with the first character and continues until
    /// the first non-device node character.
    ///
    /// Returns `None` if `text_device_node` was NULL or there was
    /// insufficient memory.
    pub fn convert_text_to_device_node(&self, text_device_node: &CStr16) -> Option<&DevicePath> {
        unsafe { (self.convert_text_to_device_node)(text_device_node.as_ptr()).as_ref() }
    }

    /// Convert a text to its binary device path representation.
    ///
    /// `text_device_path` is the text representation of a device path.
    /// Conversion starts with the first character and continues until
    /// the first non-device path character.
    ///
    /// Returns `None` if `text_device_path` was NULL or there was
    /// insufficient memory.
    pub fn convert_text_to_device_path(&self, text_device_path: &CStr16) -> Option<&DevicePath> {
        unsafe { (self.convert_text_to_device_path)(text_device_path.as_ptr()).as_ref() }
    }
}
