// SPDX-License-Identifier: MIT OR Apache-2.0

//! The UEFI device path [`Protocol`], i.e., UEFI device paths.
//!
//! This module provides high-level wrappers to work with UEFI device paths.
//! Please find additional low-level information in the
//! [device path section of `uefi-raw`].
//!
//! # Terminology: Device Paths, Device Path Instances, and Device Path Nodes
//! An open UEFI device path [`Protocol`], also called _device path_, is a
//! flexible and structured sequence of binary nodes that describes a route from
//! the UEFI root to a particular device, controller, or file.
//!
//! An entire device path can be made up of multiple device path instances,
//! and each instance is made up of multiple device path nodes. A device path
//! _may_ contain multiple device-path instances, but typical paths contain only
//! a single instance.
//!
//! Each node represents a step in the path: PCI device, partition, filesystem,
//! file path, etc. Each node represents a step in the path: PCI device,
//! partition, filesystem, file path, etc.
//!
//! Example of what a device path containing two instances (each comprised of
//! three nodes) might look like:
//!
//! ```text
//! ┌──────┬──────┬──────────────╥───────┬──────────┬────────────┐
//! │ ACPI │ PCI  │ END_INSTANCE ║ CDROM │ FILEPATH │ END_ENTIRE │
//! └──────┴──────┴──────────────╨───────┴──────────┴────────────┘
//! ↑      ↑      ↑              ↑       ↑          ↑            ↑
//! ├─Node─╨─Node─╨─────Node─────╨─Node──╨───Node───╨────Node────┤
//! ↑                            ↑                               ↑
//! ├─── DevicePathInstance ─────╨────── DevicePathInstance ─────┤
//! │                                                            │
//! └──────────────────── Entire DevicePath ─────────────────────┘
//! ```
//!
//! # Types
//!
//! To represent device paths, this module provides several types:
//!
//! * [`DevicePath`] is the root type that represents a full device
//!   path, containing one or more device path instance. It ends with an
//!   [`END_ENTIRE`] node. It implements [`Protocol`] (corresponding to
//!   `EFI_DEVICE_PATH_PROTOCOL`).
//!
//! * [`DevicePathInstance`] represents a single path instance within a
//!   device path. It ends with either an [`END_INSTANCE`] or [`END_ENTIRE`]
//!   node.
//!
//! * [`DevicePathNode`] represents a single node within a path. The
//!   node's [`device_type`] and [`sub_type`] must be examined to
//!   determine what type of data it contains.
//!
//!   Specific node types have their own structures in these submodules:
//!   * [`acpi`]
//!   * [`bios_boot_spec`]
//!   * [`end`]
//!   * [`hardware`]
//!   * [`media`]
//!   * [`messaging`]
//!
//! * [`DevicePathNodeEnum`] contains variants for references to each
//!   type of node. Call [`DevicePathNode::as_enum`] to convert from a
//!   [`DevicePathNode`] reference to a `DevicePathNodeEnum`.
//!
//! * [`DevicePathHeader`] is a header present at the start of every
//!   node. It describes the type of node as well as the node's size.
//!
//! * [`FfiDevicePath`] is an opaque type used whenever a device path
//!   pointer is passed to or from external UEFI interfaces (i.e. where
//!   the UEFI spec uses `const* EFI_DEVICE_PATH_PROTOCOL`, `*const
//!   FfiDevicePath` should be used in the Rust definition). Many of the
//!   other types in this module are DSTs, so pointers to the type are
//!   "fat" and not suitable for FFI.
//!
//! All of these types use a packed layout and may appear on any byte
//! boundary.
//!
//! Note: the API provided by this module is currently mostly limited to
//! reading existing device paths rather than constructing new ones.
//!
//! [`END_ENTIRE`]: DeviceSubType::END_ENTIRE
//! [`END_INSTANCE`]: DeviceSubType::END_INSTANCE
//! [`Protocol`]: crate::proto::Protocol
//! [`device_type`]: DevicePathNode::device_type
//! [`sub_type`]: DevicePathNode::sub_type
//! [device path section of `uefi-raw`]: uefi_raw::protocol::device_path

pub mod build;
pub mod text;
pub mod util;

mod device_path_gen;

pub use device_path_gen::{
    DevicePathNodeEnum, acpi, bios_boot_spec, end, hardware, media, messaging,
};
pub use uefi_raw::protocol::device_path::{DeviceSubType, DeviceType};

use crate::mem::PoolAllocation;
use crate::proto::{ProtocolPointer, unsafe_protocol};
use core::ffi::c_void;
use core::fmt::{self, Debug, Display, Formatter};
use core::ops::Deref;
use ptr_meta::Pointee;
use uefi_raw::protocol::device_path::DevicePathProtocol;
#[cfg(feature = "alloc")]
use {
    crate::boot::{self, OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol, SearchType},
    crate::proto::device_path::text::{AllowShortcuts, DevicePathToText, DisplayOnly},
    crate::proto::device_path::util::DevicePathUtilities,
    crate::{CString16, Identify},
    alloc::borrow::ToOwned,
    alloc::boxed::Box,
    core::mem,
};

opaque_type! {
    /// Opaque type that should be used to represent a pointer to a
    /// [`DevicePath`] or [`DevicePathNode`] in foreign function interfaces. This
    /// type produces a thin pointer, unlike [`DevicePath`] and
    /// [`DevicePathNode`].
    pub struct FfiDevicePath;
}

/// Device path allocated from UEFI pool memory.
#[derive(Debug)]
pub struct PoolDevicePath(pub(crate) PoolAllocation);

impl Deref for PoolDevicePath {
    type Target = DevicePath;

    fn deref(&self) -> &Self::Target {
        unsafe { DevicePath::from_ffi_ptr(self.0.as_ptr().as_ptr().cast()) }
    }
}

/// Device path node allocated from UEFI pool memory.
#[derive(Debug)]
pub struct PoolDevicePathNode(pub(crate) PoolAllocation);

impl Deref for PoolDevicePathNode {
    type Target = DevicePathNode;

    fn deref(&self) -> &Self::Target {
        unsafe { DevicePathNode::from_ffi_ptr(self.0.as_ptr().as_ptr().cast()) }
    }
}

/// Fixed header that appears at the start of every [`DevicePathNode`].
///
/// This type is ABI-compatible with `EFI_DEVICE_PATH_PROTOCOL`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct DevicePathHeader(DevicePathProtocol);

impl DevicePathHeader {
    /// Constructs a new [`DevicePathHeader`].
    #[must_use]
    pub const fn new(major_type: DeviceType, sub_type: DeviceSubType, length: u16) -> Self {
        Self(DevicePathProtocol {
            major_type,
            sub_type,
            length: length.to_le_bytes(),
        })
    }

    /// Returns the [`DeviceType`].
    #[must_use]
    pub const fn device_type(&self) -> DeviceType {
        self.0.major_type
    }

    /// Returns the [`DeviceSubType`].
    #[must_use]
    pub const fn sub_type(&self) -> DeviceSubType {
        self.0.sub_type
    }

    /// Returns the total length of the device path node.
    #[must_use]
    pub const fn length(&self) -> u16 {
        self.0.length()
    }
}

impl<'a> TryFrom<&'a [u8]> for &'a DevicePathHeader {
    type Error = ByteConversionError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if size_of::<DevicePathHeader>() <= bytes.len() {
            unsafe { Ok(&*bytes.as_ptr().cast::<DevicePathHeader>()) }
        } else {
            Err(ByteConversionError::InvalidLength)
        }
    }
}

/// A single node within a [`DevicePath`].
///
/// Each node starts with a [`DevicePathHeader`]. The rest of the data
/// in the node depends on the type of node. You can "cast" a node to a specific
/// one like this:
/// ```no_run
/// use uefi::proto::device_path::DevicePath;
/// use uefi::proto::device_path::media::FilePath;
///
/// let image_device_path: &DevicePath = unsafe { DevicePath::from_ffi_ptr(0x1337 as *const _) };
/// let file_path = image_device_path
///         .node_iter()
///         .find_map(|node| {
///             let node: &FilePath = node.try_into().ok()?;
///             let path = node.path_name().to_cstring16().ok()?;
///             Some(path.to_string().to_uppercase())
///         });
/// ```
/// More types are available in [`uefi::proto::device_path`]. Builder types
/// can be found in [`uefi::proto::device_path::build`]
///
/// See the [module-level documentation] for more details.
///
/// [module-level documentation]: crate::proto::device_path
#[derive(Eq, Pointee)]
#[repr(C, packed)]
pub struct DevicePathNode {
    header: DevicePathHeader,
    data: [u8],
}

impl DevicePathNode {
    /// Create a [`DevicePathNode`] reference from an opaque pointer.
    ///
    /// # Safety
    ///
    /// The input pointer must point to valid data. That data must
    /// remain valid for the lifetime `'a`, and cannot be mutated during
    /// that lifetime.
    #[must_use]
    pub unsafe fn from_ffi_ptr<'a>(ptr: *const FfiDevicePath) -> &'a Self {
        let header = unsafe { *ptr.cast::<DevicePathHeader>() };

        let data_len = usize::from(header.length()) - size_of::<DevicePathHeader>();
        unsafe { &*ptr_meta::from_raw_parts(ptr.cast(), data_len) }
    }

    /// Cast to a [`FfiDevicePath`] pointer.
    #[must_use]
    pub const fn as_ffi_ptr(&self) -> *const FfiDevicePath {
        let ptr: *const Self = self;
        ptr.cast::<FfiDevicePath>()
    }

    /// Type of device
    #[must_use]
    pub const fn device_type(&self) -> DeviceType {
        self.header.device_type()
    }

    /// Sub type of device
    #[must_use]
    pub const fn sub_type(&self) -> DeviceSubType {
        self.header.sub_type()
    }

    /// Tuple of the node's type and subtype.
    #[must_use]
    pub const fn full_type(&self) -> (DeviceType, DeviceSubType) {
        (self.device_type(), self.sub_type())
    }

    /// Size (in bytes) of the full [`DevicePathNode`], including the header.
    #[must_use]
    pub const fn length(&self) -> u16 {
        self.header.length()
    }

    /// True if this node ends an entire [`DevicePath`].
    #[must_use]
    pub fn is_end_entire(&self) -> bool {
        self.full_type() == (DeviceType::END, DeviceSubType::END_ENTIRE)
    }

    /// Returns the payload data of this node.
    #[must_use]
    pub const fn data(&self) -> &[u8] {
        &self.data
    }

    /// Convert from a generic [`DevicePathNode`] reference to an enum
    /// of more specific node types.
    pub fn as_enum(&self) -> Result<DevicePathNodeEnum<'_>, NodeConversionError> {
        DevicePathNodeEnum::try_from(self)
    }

    /// Transforms the device path node to its string representation using the
    /// [`DevicePathToText`] protocol.
    #[cfg(feature = "alloc")]
    pub fn to_string(
        &self,
        display_only: DisplayOnly,
        allow_shortcuts: AllowShortcuts,
    ) -> Result<CString16, DevicePathToTextError> {
        let to_text_protocol = open_text_protocol()?;

        to_text_protocol
            .convert_device_node_to_text(self, display_only, allow_shortcuts)
            .map(|pool_string| {
                let cstr16 = &*pool_string;
                // Another allocation; pool string is dropped. This overhead
                // is negligible. CString16 is more convenient to use.
                CString16::from(cstr16)
            })
            .map_err(|_| DevicePathToTextError::OutOfMemory)
    }
}

impl Debug for DevicePathNode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("DevicePathNode")
            .field("header", &self.header)
            .field("data", &&self.data)
            .finish()
    }
}

impl PartialEq for DevicePathNode {
    fn eq(&self, other: &Self) -> bool {
        self.header == other.header && self.data == other.data
    }
}

impl<'a> TryFrom<&'a [u8]> for &'a DevicePathNode {
    type Error = ByteConversionError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let dp = <&DevicePathHeader>::try_from(bytes)?;
        if usize::from(dp.length()) <= bytes.len() {
            unsafe { Ok(DevicePathNode::from_ffi_ptr(bytes.as_ptr().cast())) }
        } else {
            Err(ByteConversionError::InvalidLength)
        }
    }
}

/// A single device path instance that ends with either an [`END_INSTANCE`]
/// or [`END_ENTIRE`] node. Use [`DevicePath::instance_iter`] to get the
/// path instances in a [`DevicePath`].
///
/// See the [module-level documentation] for more details.
///
/// [`END_ENTIRE`]: DeviceSubType::END_ENTIRE
/// [`END_INSTANCE`]: DeviceSubType::END_INSTANCE
/// [module-level documentation]: crate::proto::device_path
#[repr(C, packed)]
#[derive(Eq, Pointee)]
pub struct DevicePathInstance {
    data: [u8],
}

impl DevicePathInstance {
    /// Get an iterator over the [`DevicePathNodes`] in this
    /// instance. Iteration ends when any [`DeviceType::END`] node is
    /// reached.
    ///
    /// [`DevicePathNodes`]: DevicePathNode
    #[must_use]
    pub const fn node_iter(&self) -> DevicePathNodeIterator<'_> {
        DevicePathNodeIterator {
            nodes: &self.data,
            stop_condition: StopCondition::AnyEndNode,
        }
    }

    /// Returns a slice of the underlying bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Returns a boxed copy of that value.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn to_boxed(&self) -> Box<Self> {
        let data = self.data.to_owned();
        let data = data.into_boxed_slice();
        unsafe { mem::transmute(data) }
    }
}

impl Debug for DevicePathInstance {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("DevicePathInstance")
            .field("data", &&self.data)
            .finish()
    }
}

impl PartialEq for DevicePathInstance {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

#[cfg(feature = "alloc")]
impl ToOwned for DevicePathInstance {
    type Owned = Box<Self>;

    fn to_owned(&self) -> Self::Owned {
        self.to_boxed()
    }
}

/// High-level representation of the UEFI [device path protocol].
///
/// This type represents an entire device path, possibly consisting of multiple
/// [`DevicePathInstance`]s and [`DevicePathNode`]s.
///
/// See the [module-level documentation] for more details.
///
/// # Usage
/// This type implements [`Protocol`] and therefore can be used on any
/// device handle to obtain generic path/location information concerning the
/// physical device or logical device. If the handle does not logically map to a
/// physical device, the handle may not necessarily support the device path
/// protocol. The device path describes the location of the device the handle is
/// for. The size of the Device Path can be determined from the structures that
/// make up the Device Path.
///
/// # Example
/// ```rust,no_run
/// use uefi::Handle;
/// use uefi::boot::{open_protocol_exclusive, ScopedProtocol};
/// use uefi::proto::device_path::DevicePath;
/// use uefi::proto::device_path::text::{AllowShortcuts, DisplayOnly};
/// use uefi::proto::loaded_image::LoadedImage;
///
/// fn open_device_path(image_handle: Handle) {
///     let loaded_image = open_protocol_exclusive::<LoadedImage>(image_handle).unwrap();
///     let device_handle = loaded_image.device().unwrap();
///     let device_path: ScopedProtocol<DevicePath>
///         = open_protocol_exclusive::<DevicePath>(device_handle).unwrap();
///     log::debug!(
///         "Device path: {}",
///         device_path.to_string(DisplayOnly(true), AllowShortcuts(true)).unwrap()
///     );
/// }
/// ```
///
/// [module-level documentation]: crate::proto::device_path
/// [`END_ENTIRE`]: DeviceSubType::END_ENTIRE
/// [`DevicePathProtocol`]: uefi_raw::protocol::device_path::DevicePathProtocol
/// [`Protocol`]: uefi::proto::Protocol
/// [device path protocol]: uefi_raw::protocol::device_path
#[repr(C, packed)]
#[unsafe_protocol(DevicePathProtocol::GUID)]
#[derive(Eq, Pointee)]
pub struct DevicePath {
    data: [u8],
}

impl ProtocolPointer for DevicePath {
    unsafe fn ptr_from_ffi(ptr: *const c_void) -> *const Self {
        ptr_meta::from_raw_parts(ptr.cast(), unsafe { Self::size_in_bytes_from_ptr(ptr) })
    }

    unsafe fn mut_ptr_from_ffi(ptr: *mut c_void) -> *mut Self {
        ptr_meta::from_raw_parts_mut(ptr.cast(), unsafe { Self::size_in_bytes_from_ptr(ptr) })
    }
}

impl DevicePath {
    /// Calculate the size in bytes of the entire `DevicePath` starting
    /// at `ptr`. This adds up each node's length, including the
    /// end-entire node.
    unsafe fn size_in_bytes_from_ptr(ptr: *const c_void) -> usize {
        let mut ptr = ptr.cast::<u8>();
        let mut total_size_in_bytes: usize = 0;
        loop {
            let node = unsafe { DevicePathNode::from_ffi_ptr(ptr.cast::<FfiDevicePath>()) };
            let node_size_in_bytes = usize::from(node.length());
            total_size_in_bytes += node_size_in_bytes;
            if node.is_end_entire() {
                break;
            }
            ptr = unsafe { ptr.add(node_size_in_bytes) };
        }

        total_size_in_bytes
    }

    /// Calculate the size in bytes of the entire `DevicePath` starting
    /// at `bytes`. This adds up each node's length, including the
    /// end-entire node.
    ///
    /// # Errors
    ///
    /// The [`ByteConversionError::InvalidLength`] error will be returned
    /// when the length of the given bytes slice cannot contain the full
    /// [`DevicePath`] represented by the slice.
    fn size_in_bytes_from_slice(mut bytes: &[u8]) -> Result<usize, ByteConversionError> {
        let max_size_in_bytes = bytes.len();
        let mut total_size_in_bytes: usize = 0;
        loop {
            let node = <&DevicePathNode>::try_from(bytes)?;
            let node_size_in_bytes = usize::from(node.length());
            total_size_in_bytes += node_size_in_bytes;
            // Length of last processed node extends past the bytes slice.
            if total_size_in_bytes > max_size_in_bytes {
                return Err(ByteConversionError::InvalidLength);
            }
            if node.is_end_entire() {
                break;
            }
            bytes = &bytes[node_size_in_bytes..];
        }

        Ok(total_size_in_bytes)
    }

    /// Create a [`DevicePath`] reference from an opaque pointer.
    ///
    /// # Safety
    ///
    /// The input pointer must point to valid data. That data must
    /// remain valid for the lifetime `'a`, and cannot be mutated during
    /// that lifetime.
    #[must_use]
    pub unsafe fn from_ffi_ptr<'a>(ptr: *const FfiDevicePath) -> &'a Self {
        unsafe { &*Self::ptr_from_ffi(ptr.cast::<c_void>()) }
    }

    /// Cast to a [`FfiDevicePath`] pointer.
    #[must_use]
    pub const fn as_ffi_ptr(&self) -> *const FfiDevicePath {
        let p = self as *const Self;
        p.cast()
    }

    /// Get an iterator over the [`DevicePathInstance`]s in this path.
    #[must_use]
    pub const fn instance_iter(&self) -> DevicePathInstanceIterator<'_> {
        DevicePathInstanceIterator {
            remaining_path: Some(self),
        }
    }

    /// Get an iterator over the [`DevicePathNode`]s starting at
    /// `self`. Iteration ends when a path is reached where
    /// [`is_end_entire`][DevicePathNode::is_end_entire] is true. That ending
    /// path is not returned by the iterator.
    #[must_use]
    pub const fn node_iter(&self) -> DevicePathNodeIterator<'_> {
        DevicePathNodeIterator {
            nodes: &self.data,
            stop_condition: StopCondition::EndEntireNode,
        }
    }

    /// Returns a slice of the underlying bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Returns a boxed copy of that value.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn to_boxed(&self) -> Box<Self> {
        let data = self.data.to_owned();
        let data = data.into_boxed_slice();
        unsafe { mem::transmute(data) }
    }

    /// Transforms the device path to its string representation using the
    /// [`DevicePathToText`] protocol.
    #[cfg(feature = "alloc")]
    pub fn to_string(
        &self,
        display_only: DisplayOnly,
        allow_shortcuts: AllowShortcuts,
    ) -> Result<CString16, DevicePathToTextError> {
        let to_text_protocol = open_text_protocol()?;

        to_text_protocol
            .convert_device_path_to_text(self, display_only, allow_shortcuts)
            .map(|pool_string| {
                let cstr16 = &*pool_string;
                // Another allocation; pool string is dropped. This overhead
                // is negligible. CString16 is more convenient to use.
                CString16::from(cstr16)
            })
            .map_err(|_| DevicePathToTextError::OutOfMemory)
    }

    /// Allocates and returns a new [`DevicePath`] by copying this one and appending the given `right` path.
    #[cfg(feature = "alloc")]
    pub fn append_path(&self, right: &Self) -> Result<PoolDevicePath, DevicePathUtilitiesError> {
        open_utility_protocol()?
            .append_path(self, right)
            .map_err(|_| DevicePathUtilitiesError::OutOfMemory)
    }

    /// Allocates and returns a new [`DevicePath`] by copying this one and appending the given `right` node.
    #[cfg(feature = "alloc")]
    pub fn append_node(
        &self,
        right: &DevicePathNode,
    ) -> Result<PoolDevicePath, DevicePathUtilitiesError> {
        open_utility_protocol()?
            .append_node(self, right)
            .map_err(|_| DevicePathUtilitiesError::OutOfMemory)
    }
}

impl Debug for DevicePath {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("DevicePath")
            .field("data", &&self.data)
            .finish()
    }
}

impl PartialEq for DevicePath {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<'a> TryFrom<&'a [u8]> for &'a DevicePath {
    type Error = ByteConversionError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let len = DevicePath::size_in_bytes_from_slice(bytes)?;
        unsafe { Ok(&*ptr_meta::from_raw_parts(bytes.as_ptr().cast(), len)) }
    }
}

#[cfg(feature = "alloc")]
impl ToOwned for DevicePath {
    type Owned = Box<Self>;

    fn to_owned(&self) -> Self::Owned {
        self.to_boxed()
    }
}

/// Iterator over the [`DevicePathInstance`]s in a [`DevicePath`].
///
/// This struct is returned by [`DevicePath::instance_iter`].
#[derive(Debug)]
pub struct DevicePathInstanceIterator<'a> {
    remaining_path: Option<&'a DevicePath>,
}

impl<'a> Iterator for DevicePathInstanceIterator<'a> {
    type Item = &'a DevicePathInstance;

    fn next(&mut self) -> Option<Self::Item> {
        let remaining_path = self.remaining_path?;

        let mut instance_size: usize = 0;

        // Find the end of the instance, which can be either kind of end
        // node (end-instance or end-entire). Count the number of bytes
        // up to and including that end node.
        let node_iter = DevicePathNodeIterator {
            nodes: &remaining_path.data,
            stop_condition: StopCondition::NoMoreNodes,
        };
        for node in node_iter {
            instance_size += usize::from(node.length());
            if node.device_type() == DeviceType::END {
                break;
            }
        }

        let (head, rest) = remaining_path.data.split_at(instance_size);

        if rest.is_empty() {
            self.remaining_path = None;
        } else {
            self.remaining_path = unsafe {
                Some(&*ptr_meta::from_raw_parts(
                    rest.as_ptr().cast::<()>(),
                    rest.len(),
                ))
            };
        }

        unsafe {
            Some(&*ptr_meta::from_raw_parts(
                head.as_ptr().cast::<()>(),
                head.len(),
            ))
        }
    }
}

#[derive(Debug)]
enum StopCondition {
    AnyEndNode,
    EndEntireNode,
    NoMoreNodes,
}

/// Iterator over [`DevicePathNode`]s.
///
/// This struct is returned by [`DevicePath::node_iter`] and
/// [`DevicePathInstance::node_iter`].
#[derive(Debug)]
pub struct DevicePathNodeIterator<'a> {
    nodes: &'a [u8],
    stop_condition: StopCondition,
}

impl<'a> Iterator for DevicePathNodeIterator<'a> {
    type Item = &'a DevicePathNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.nodes.is_empty() {
            return None;
        }

        let node =
            unsafe { DevicePathNode::from_ffi_ptr(self.nodes.as_ptr().cast::<FfiDevicePath>()) };

        // Check if an early stop condition has been reached.
        let stop = match self.stop_condition {
            StopCondition::AnyEndNode => node.device_type() == DeviceType::END,
            StopCondition::EndEntireNode => node.is_end_entire(),
            StopCondition::NoMoreNodes => false,
        };

        if stop {
            // Clear the remaining node data so that future calls to
            // next() immediately return `None`.
            self.nodes = &[];
            None
        } else {
            // Advance to next node.
            let node_size = usize::from(node.length());
            self.nodes = &self.nodes[node_size..];
            Some(node)
        }
    }
}

/// Error returned when attempting to convert from a `&[u8]` to a
/// [`DevicePath`] type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ByteConversionError {
    /// The length of the given slice is not valid for its [`DevicePath`] type.
    InvalidLength,
}

/// Error returned when converting from a [`DevicePathNode`] to a more
/// specific node type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodeConversionError {
    /// The requested node type does not match the actual node type.
    DifferentType,

    /// The length of the node data is not valid for its type.
    InvalidLength,

    /// The node type is not currently supported.
    UnsupportedType,
}

/// Loaded Image Device Path [`Protocol`].
///
/// Protocol for accessing the device path that was passed in to [`load_image`]
/// when loading a PE/COFF image.
///
/// The layout of this type is the same as a [`DevicePath`].
///
/// [`load_image`]: crate::boot::load_image
/// [`Protocol`]: uefi::proto::Protocol
#[repr(transparent)]
#[unsafe_protocol("bc62157e-3e33-4fec-9920-2d3b36d750df")]
#[derive(Debug, Pointee)]
pub struct LoadedImageDevicePath(DevicePath);

impl ProtocolPointer for LoadedImageDevicePath {
    unsafe fn ptr_from_ffi(ptr: *const c_void) -> *const Self {
        ptr_meta::from_raw_parts(ptr.cast(), unsafe {
            DevicePath::size_in_bytes_from_ptr(ptr)
        })
    }

    unsafe fn mut_ptr_from_ffi(ptr: *mut c_void) -> *mut Self {
        ptr_meta::from_raw_parts_mut(ptr.cast(), unsafe {
            DevicePath::size_in_bytes_from_ptr(ptr)
        })
    }
}

impl Deref for LoadedImageDevicePath {
    type Target = DevicePath;

    fn deref(&self) -> &DevicePath {
        &self.0
    }
}

/// Errors that may happen when a device path is transformed to a string
/// representation using:
/// - [`DevicePath::to_string`]
/// - [`DevicePathNode::to_string`]
#[derive(Debug)]
pub enum DevicePathToTextError {
    /// Can't locate a handle buffer with handles associated with the
    /// [`DevicePathToText`] protocol.
    CantLocateHandleBuffer(crate::Error),
    /// There is no handle supporting the [`DevicePathToText`] protocol.
    NoHandle,
    /// The handle supporting the [`DevicePathToText`] protocol exists but it
    /// could not be opened.
    CantOpenProtocol(crate::Error),
    /// Failed to allocate pool memory.
    OutOfMemory,
}

impl Display for DevicePathToTextError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl core::error::Error for DevicePathToTextError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::CantLocateHandleBuffer(e) => Some(e),
            Self::CantOpenProtocol(e) => Some(e),
            _ => None,
        }
    }
}

/// Helper function to open the [`DevicePathToText`] protocol using the boot
/// services.
#[cfg(feature = "alloc")]
fn open_text_protocol() -> Result<ScopedProtocol<DevicePathToText>, DevicePathToTextError> {
    let &handle = boot::locate_handle_buffer(SearchType::ByProtocol(&DevicePathToText::GUID))
        .map_err(DevicePathToTextError::CantLocateHandleBuffer)?
        .first()
        .ok_or(DevicePathToTextError::NoHandle)?;

    unsafe {
        boot::open_protocol::<DevicePathToText>(
            OpenProtocolParams {
                handle,
                agent: boot::image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )
    }
    .map_err(DevicePathToTextError::CantOpenProtocol)
}

/// Errors that may occur when working with the [`DevicePathUtilities`] protocol.
///
/// These errors are typically encountered during operations involving device
/// paths, such as appending or manipulating path segments.
#[derive(Debug)]
pub enum DevicePathUtilitiesError {
    /// Can't locate a handle buffer with handles associated with the
    /// [`DevicePathUtilities`] protocol.
    CantLocateHandleBuffer(crate::Error),
    /// No handle supporting the [`DevicePathUtilities`] protocol was found.
    NoHandle,
    /// The handle supporting the [`DevicePathUtilities`] protocol exists but
    /// it could not be opened.
    CantOpenProtocol(crate::Error),
    /// Memory allocation failed during device path operations.
    OutOfMemory,
}

impl Display for DevicePathUtilitiesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl core::error::Error for DevicePathUtilitiesError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::CantLocateHandleBuffer(e) => Some(e),
            Self::CantOpenProtocol(e) => Some(e),
            _ => None,
        }
    }
}

/// Helper function to open the [`DevicePathUtilities`] protocol using the boot
/// services.
#[cfg(feature = "alloc")]
fn open_utility_protocol() -> Result<ScopedProtocol<DevicePathUtilities>, DevicePathUtilitiesError>
{
    let &handle = boot::locate_handle_buffer(SearchType::ByProtocol(&DevicePathToText::GUID))
        .map_err(DevicePathUtilitiesError::CantLocateHandleBuffer)?
        .first()
        .ok_or(DevicePathUtilitiesError::NoHandle)?;

    unsafe {
        boot::open_protocol::<DevicePathUtilities>(
            OpenProtocolParams {
                handle,
                agent: boot::image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )
    }
    .map_err(DevicePathUtilitiesError::CantOpenProtocol)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    /// Create a node to `path` from raw data.
    fn add_node(path: &mut Vec<u8>, device_type: u8, sub_type: u8, node_data: &[u8]) {
        path.push(device_type);
        path.push(sub_type);
        path.extend(
            u16::try_from(size_of::<DevicePathHeader>() + node_data.len())
                .unwrap()
                .to_le_bytes(),
        );
        path.extend(node_data);
    }

    /// Create a test device path list as raw bytes.
    fn create_raw_device_path() -> Vec<u8> {
        let mut raw_data = Vec::new();

        // First path instance.
        add_node(&mut raw_data, 0xa0, 0xb0, &[10, 11]);
        add_node(&mut raw_data, 0xa1, 0xb1, &[20, 21, 22, 23]);
        add_node(
            &mut raw_data,
            DeviceType::END.0,
            DeviceSubType::END_INSTANCE.0,
            &[],
        );
        // Second path instance.
        add_node(&mut raw_data, 0xa2, 0xb2, &[30, 31]);
        add_node(&mut raw_data, 0xa3, 0xb3, &[40, 41, 42, 43]);
        add_node(
            &mut raw_data,
            DeviceType::END.0,
            DeviceSubType::END_ENTIRE.0,
            &[],
        );

        raw_data
    }

    /// Check that `node` has the expected content.
    fn check_node(node: &DevicePathNode, device_type: u8, sub_type: u8, node_data: &[u8]) {
        assert_eq!(node.device_type().0, device_type);
        assert_eq!(node.sub_type().0, sub_type);
        assert_eq!(
            node.length(),
            u16::try_from(size_of::<DevicePathHeader>() + node_data.len()).unwrap()
        );
        assert_eq!(&node.data, node_data);
    }

    #[test]
    fn test_device_path_nodes() {
        let raw_data = create_raw_device_path();
        let dp = unsafe { DevicePath::from_ffi_ptr(raw_data.as_ptr().cast()) };

        // Check that the size is the sum of the nodes' lengths.
        assert_eq!(size_of_val(dp), 6 + 8 + 4 + 6 + 8 + 4);

        // Check the list's node iter.
        let nodes: Vec<_> = dp.node_iter().collect();
        check_node(nodes[0], 0xa0, 0xb0, &[10, 11]);
        check_node(nodes[1], 0xa1, 0xb1, &[20, 21, 22, 23]);
        check_node(
            nodes[2],
            DeviceType::END.0,
            DeviceSubType::END_INSTANCE.0,
            &[],
        );
        check_node(nodes[3], 0xa2, 0xb2, &[30, 31]);
        check_node(nodes[4], 0xa3, 0xb3, &[40, 41, 42, 43]);
        // The end-entire node is not returned by the iterator.
        assert_eq!(nodes.len(), 5);
    }

    #[test]
    fn test_device_path_instances() {
        let raw_data = create_raw_device_path();
        let dp = unsafe { DevicePath::from_ffi_ptr(raw_data.as_ptr().cast()) };

        // Check the list's instance iter.
        let mut iter = dp.instance_iter();
        let mut instance = iter.next().unwrap();
        assert_eq!(size_of_val(instance), 6 + 8 + 4);

        // Check the first instance's node iter.
        let nodes: Vec<_> = instance.node_iter().collect();
        check_node(nodes[0], 0xa0, 0xb0, &[10, 11]);
        check_node(nodes[1], 0xa1, 0xb1, &[20, 21, 22, 23]);
        // The end node is not returned by the iterator.
        assert_eq!(nodes.len(), 2);

        // Check second instance.
        instance = iter.next().unwrap();
        assert_eq!(size_of_val(instance), 6 + 8 + 4);

        let nodes: Vec<_> = instance.node_iter().collect();
        check_node(nodes[0], 0xa2, 0xb2, &[30, 31]);
        check_node(nodes[1], 0xa3, 0xb3, &[40, 41, 42, 43]);
        // The end node is not returned by the iterator.
        assert_eq!(nodes.len(), 2);

        // Only two instances.
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_to_owned() {
        // Relevant assertion to verify the transmute is fine.
        assert_eq!(size_of::<&DevicePath>(), size_of::<&[u8]>());

        let raw_data = create_raw_device_path();
        let dp = unsafe { DevicePath::from_ffi_ptr(raw_data.as_ptr().cast()) };

        // Relevant assertion to verify the transmute is fine.
        assert_eq!(size_of_val(dp), size_of_val(&dp.data));

        let owned_dp = dp.to_owned();
        let owned_dp_ref = &*owned_dp;
        assert_eq!(owned_dp_ref, dp)
    }

    #[test]
    fn test_device_path_node_from_bytes() {
        let mut raw_data = Vec::new();
        let node = [0xa0, 0xb0];
        let node_data = &[10, 11];

        // Raw data is less than size of a [`DevicePathNode`].
        raw_data.push(node[0]);
        assert!(<&DevicePathNode>::try_from(raw_data.as_slice()).is_err());

        // Raw data is long enough to hold a [`DevicePathNode`].
        raw_data.push(node[1]);
        raw_data.extend(
            u16::try_from(size_of::<DevicePathHeader>() + node_data.len())
                .unwrap()
                .to_le_bytes(),
        );
        raw_data.extend(node_data);
        let dp = <&DevicePathNode>::try_from(raw_data.as_slice()).unwrap();

        // Relevant assertions to verify the conversion is fine.
        assert_eq!(size_of_val(dp), 6);
        check_node(dp, 0xa0, 0xb0, &[10, 11]);

        // [`DevicePathNode`] data length exceeds the raw_data slice.
        raw_data[2] += 1;
        assert!(<&DevicePathNode>::try_from(raw_data.as_slice()).is_err());
    }

    #[test]
    fn test_device_path_nodes_from_bytes() {
        let raw_data = create_raw_device_path();
        let dp = <&DevicePath>::try_from(raw_data.as_slice()).unwrap();

        // Check that the size is the sum of the nodes' lengths.
        assert_eq!(size_of_val(dp), 6 + 8 + 4 + 6 + 8 + 4);

        // Check the list's node iter.
        let nodes: Vec<_> = dp.node_iter().collect();
        check_node(nodes[0], 0xa0, 0xb0, &[10, 11]);
        check_node(nodes[1], 0xa1, 0xb1, &[20, 21, 22, 23]);
        check_node(
            nodes[2],
            DeviceType::END.0,
            DeviceSubType::END_INSTANCE.0,
            &[],
        );
        check_node(nodes[3], 0xa2, 0xb2, &[30, 31]);
        check_node(nodes[4], 0xa3, 0xb3, &[40, 41, 42, 43]);
        // The end-entire node is not returned by the iterator.
        assert_eq!(nodes.len(), 5);
    }

    /// Test converting from `&DevicePathNode` to a specific node type.
    #[test]
    fn test_specific_node_from_device_path_node() {
        let mut raw_data = Vec::new();
        add_node(
            &mut raw_data,
            DeviceType::END.0,
            DeviceSubType::END_INSTANCE.0,
            &[],
        );
        let node = <&DevicePathNode>::try_from(raw_data.as_slice()).unwrap();

        assert!(<&end::Instance>::try_from(node).is_ok());
        assert_eq!(
            <&end::Entire>::try_from(node).unwrap_err(),
            NodeConversionError::DifferentType
        );
    }
}
