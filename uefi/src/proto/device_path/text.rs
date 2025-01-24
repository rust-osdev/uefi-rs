//! `DevicePathToText` and `DevicePathFromText` Protocol

// Note on return types: the specification of the conversion functions
// is a little unusual in that they return a pointer rather than
// `EFI_STATUS`. A NULL pointer is used to indicate an error, and the
// spec says that will only happen if the input pointer is null (which
// can't happen here since we use references as input, not pointers), or
// if there is insufficient memory. So we treat any NULL output as an
// `OUT_OF_RESOURCES` error.

use crate::mem::PoolAllocation;
use crate::proto::device_path::{DevicePath, DevicePathNode};
use crate::proto::unsafe_protocol;
use crate::{CStr16, Char16, Result, Status};
use core::ops::Deref;
use core::ptr::NonNull;
use uefi_raw::protocol::device_path::{DevicePathFromTextProtocol, DevicePathToTextProtocol};

/// This struct is a wrapper of `display_only` parameter
/// used by Device Path to Text protocol.
///
/// The `display_only` parameter controls whether the longer
/// (parseable)  or shorter (display-only) form of the conversion
/// is used. If `display_only` is TRUE, then the shorter text
/// representation of the display node is used, where applicable.
/// If `display_only` is FALSE, then the longer text representation
/// of the display node is used.
#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Copy, Debug)]
pub struct AllowShortcuts(pub bool);

/// Wrapper for a string internally allocated from
/// UEFI boot services memory.
#[derive(Debug)]
pub struct PoolString(PoolAllocation);

impl PoolString {
    fn new(text: *const Char16) -> Result<Self> {
        NonNull::new(text.cast_mut())
            .map(|p| Self(PoolAllocation::new(p.cast())))
            .ok_or(Status::OUT_OF_RESOURCES.into())
    }
}

impl Deref for PoolString {
    type Target = CStr16;

    fn deref(&self) -> &Self::Target {
        unsafe { CStr16::from_ptr(self.0.as_ptr().as_ptr().cast()) }
    }
}

/// Device Path to Text protocol.
///
/// This protocol provides common utility functions for converting device
/// nodes and device paths to a text representation.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(DevicePathToTextProtocol::GUID)]
pub struct DevicePathToText(DevicePathToTextProtocol);

impl DevicePathToText {
    /// Convert a device node to its text representation.
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
        let text_device_node = unsafe {
            (self.0.convert_device_node_to_text)(
                device_node.as_ffi_ptr().cast(),
                display_only.0,
                allow_shortcuts.0,
            )
        };
        PoolString::new(text_device_node.cast())
    }

    /// Convert a device path to its text representation.
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
        let text_device_path = unsafe {
            (self.0.convert_device_path_to_text)(
                device_path.as_ffi_ptr().cast(),
                display_only.0,
                allow_shortcuts.0,
            )
        };
        PoolString::new(text_device_path.cast())
    }
}

/// Device Path from Text protocol.
///
/// This protocol provides common utilities for converting text to
/// device paths and device nodes.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol("05c99a21-c70f-4ad2-8a5f-35df3343f51e")]
pub struct DevicePathFromText(DevicePathFromTextProtocol);

impl DevicePathFromText {
    /// Convert text to the binary representation of a device node.
    ///
    /// `text_device_node` is the text representation of a device node.
    /// Conversion starts with the first character and continues until
    /// the first non-device node character.
    ///
    /// Returns an [`OUT_OF_RESOURCES`] error if there is insufficient
    /// memory for the conversion.
    ///
    /// [`OUT_OF_RESOURCES`]: Status::OUT_OF_RESOURCES
    pub fn convert_text_to_device_node(
        &self,
        text_device_node: &CStr16,
    ) -> Result<&DevicePathNode> {
        unsafe {
            let ptr = (self.0.convert_text_to_device_node)(text_device_node.as_ptr().cast());
            if ptr.is_null() {
                Err(Status::OUT_OF_RESOURCES.into())
            } else {
                Ok(DevicePathNode::from_ffi_ptr(ptr.cast()))
            }
        }
    }

    /// Convert a text to its binary device path representation.
    ///
    /// `text_device_path` is the text representation of a device path.
    /// Conversion starts with the first character and continues until
    /// the first non-device path character.
    ///
    /// Returns an [`OUT_OF_RESOURCES`] error if there is insufficient
    /// memory for the conversion.
    ///
    /// [`OUT_OF_RESOURCES`]: Status::OUT_OF_RESOURCES
    pub fn convert_text_to_device_path(&self, text_device_path: &CStr16) -> Result<&DevicePath> {
        unsafe {
            let ptr = (self.0.convert_text_to_device_path)(text_device_path.as_ptr().cast());
            if ptr.is_null() {
                Err(Status::OUT_OF_RESOURCES.into())
            } else {
                Ok(DevicePath::from_ffi_ptr(ptr.cast()))
            }
        }
    }
}
