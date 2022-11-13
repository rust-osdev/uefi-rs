//! `DevicePathToText` and `DevicePathFromText` Protocol

// Note on return types: the specification of the conversion functions
// is a little unusual in that they return a pointer rather than
// `EFI_STATUS`. A NULL pointer is used to indicate an error, and the
// spec says that will only happen if the input pointer is null (which
// can't happen here since we use references as input, not pointers), or
// if there is insufficient memory. So we treat any NULL output as an
// `OUT_OF_RESOURCES` error.

use crate::{
    proto::device_path::{DevicePath, DevicePathNode, FfiDevicePath},
    proto::Protocol,
    table::boot::BootServices,
    unsafe_guid, CStr16, Char16, Result, Status,
};
use core::ops::Deref;

/// This struct is a wrapper of `display_only` parameter
/// used by Device Path to Text protocol.
///
/// The `display_only` parameter controls whether the longer
/// (parseable)  or shorter (display-only) form of the conversion
/// is used. If `display_only` is TRUE, then the shorter text
/// representation of the display node is used, where applicable.
/// If `display_only` is FALSE, then the longer text representation
/// of the display node is used.
#[derive(Clone, Copy)]
pub struct DisplayOnly(pub bool);

/// This struct is a wrapper of `allow_shortcuts` parameter
/// used by Device Path to Text protocol.
///
/// The `allow_shortcuts` is FALSE, then the shortcut forms of
/// text representation for a device node cannot be used. A
/// shortcut form is one which uses information other than the
/// type or subtype. If `allow_shortcuts is TRUE, then the
/// shortcut forms of text representation for a device node
/// can be used, where applicable.
#[derive(Clone, Copy)]
pub struct AllowShortcuts(pub bool);

/// Wrapper for a string internally allocated from
/// UEFI boot services memory.
pub struct PoolString<'a> {
    boot_services: &'a BootServices,
    text: *const Char16,
}

impl<'a> PoolString<'a> {
    fn new(boot_services: &'a BootServices, text: *const Char16) -> Result<Self> {
        if text.is_null() {
            Err(Status::OUT_OF_RESOURCES.into())
        } else {
            Ok(Self {
                boot_services,
                text,
            })
        }
    }
}

impl<'a> Deref for PoolString<'a> {
    type Target = CStr16;

    fn deref(&self) -> &Self::Target {
        unsafe { CStr16::from_ptr(self.text) }
    }
}

impl Drop for PoolString<'_> {
    fn drop(&mut self) {
        let addr = self.text as *mut u8;
        self.boot_services
            .free_pool(addr)
            .expect("Failed to free pool [{addr:#?}]");
    }
}

/// Device Path to Text protocol.
///
/// This protocol provides common utility functions for converting device
/// nodes and device paths to a text representation.
#[repr(C)]
#[unsafe_guid("8b843e20-8132-4852-90cc-551a4e4a7f1c")]
#[derive(Protocol)]
pub struct DevicePathToText {
    convert_device_node_to_text: unsafe extern "efiapi" fn(
        device_node: *const FfiDevicePath,
        display_only: bool,
        allow_shortcuts: bool,
    ) -> *const Char16,
    convert_device_path_to_text: unsafe extern "efiapi" fn(
        device_path: *const FfiDevicePath,
        display_only: bool,
        allow_shortcuts: bool,
    ) -> *const Char16,
}

impl DevicePathToText {
    /// Convert a device node to its text representation.
    ///
    /// Returns an [`OUT_OF_RESOURCES`] error if there is unsufficient
    /// memory for the conversion.
    ///
    /// [`OUT_OF_RESOURCES`]: Status::OUT_OF_RESOURCES
    pub fn convert_device_node_to_text<'boot>(
        &self,
        boot_services: &'boot BootServices,
        device_node: &DevicePathNode,
        display_only: DisplayOnly,
        allow_shortcuts: AllowShortcuts,
    ) -> Result<PoolString<'boot>> {
        let text_device_node = unsafe {
            (self.convert_device_node_to_text)(
                device_node.as_ffi_ptr(),
                display_only.0,
                allow_shortcuts.0,
            )
        };
        PoolString::new(boot_services, text_device_node)
    }

    /// Convert a device path to its text representation.
    ///
    /// Returns an [`OUT_OF_RESOURCES`] error if there is unsufficient
    /// memory for the conversion.
    ///
    /// [`OUT_OF_RESOURCES`]: Status::OUT_OF_RESOURCES
    pub fn convert_device_path_to_text<'boot>(
        &self,
        boot_services: &'boot BootServices,
        device_path: &DevicePath,
        display_only: DisplayOnly,
        allow_shortcuts: AllowShortcuts,
    ) -> Result<PoolString<'boot>> {
        let text_device_path = unsafe {
            (self.convert_device_path_to_text)(
                device_path.as_ffi_ptr(),
                display_only.0,
                allow_shortcuts.0,
            )
        };
        PoolString::new(boot_services, text_device_path)
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
        unsafe extern "efiapi" fn(text_device_node: *const Char16) -> *const FfiDevicePath,
    convert_text_to_device_path:
        unsafe extern "efiapi" fn(text_device_path: *const Char16) -> *const FfiDevicePath,
}

impl DevicePathFromText {
    /// Convert text to the binary representation of a device node.
    ///
    /// `text_device_node` is the text representation of a device node.
    /// Conversion starts with the first character and continues until
    /// the first non-device node character.
    ///
    /// Returns an [`OUT_OF_RESOURCES`] error if there is unsufficient
    /// memory for the conversion.
    ///
    /// [`OUT_OF_RESOURCES`]: Status::OUT_OF_RESOURCES
    pub fn convert_text_to_device_node(
        &self,
        text_device_node: &CStr16,
    ) -> Result<&DevicePathNode> {
        unsafe {
            let ptr = (self.convert_text_to_device_node)(text_device_node.as_ptr());
            if ptr.is_null() {
                Err(Status::OUT_OF_RESOURCES.into())
            } else {
                Ok(DevicePathNode::from_ffi_ptr(ptr))
            }
        }
    }

    /// Convert a text to its binary device path representation.
    ///
    /// `text_device_path` is the text representation of a device path.
    /// Conversion starts with the first character and continues until
    /// the first non-device path character.
    ///
    /// Returns an [`OUT_OF_RESOURCES`] error if there is unsufficient
    /// memory for the conversion.
    ///
    /// [`OUT_OF_RESOURCES`]: Status::OUT_OF_RESOURCES
    pub fn convert_text_to_device_path(&self, text_device_path: &CStr16) -> Result<&DevicePath> {
        unsafe {
            let ptr = (self.convert_text_to_device_path)(text_device_path.as_ptr());
            if ptr.is_null() {
                Err(Status::OUT_OF_RESOURCES.into())
            } else {
                Ok(DevicePath::from_ffi_ptr(ptr))
            }
        }
    }
}
