// SPDX-License-Identifier: MIT OR Apache-2.0

//! Protocol with utility functions for working with device paths.

use super::{DevicePath, DevicePathNode, PoolDevicePath};
use crate::mem::PoolAllocation;
use core::ptr::NonNull;
use uefi_macros::unsafe_protocol;
use uefi_raw::protocol::device_path::DevicePathUtilitiesProtocol;
use uefi_raw::Status;

/// Protocol with utility functions for working with device paths.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(DevicePathUtilitiesProtocol::GUID)]
pub struct DevicePathUtilities(DevicePathUtilitiesProtocol);

impl DevicePathUtilities {
    /// Retrieves the size of the specified device path in bytes, including the
    /// end-of-device-path node.
    ///
    /// # Parameters
    /// - `device_path`: A reference to the [`DevicePath`] whose size is to be determined.
    ///
    /// # Returns
    /// The size of the specified device path in bytes.
    #[must_use]
    pub fn get_size(&self, device_path: &DevicePath) -> usize {
        unsafe { (self.0.get_device_path_size)(device_path.as_ffi_ptr().cast()) }
    }

    /// Creates a new device path by appending the second device path to the first.
    ///
    /// # Parameters
    /// - `path0`: A reference to the base device path.
    /// - `path1`: A reference to the device path to append.
    ///
    /// # Returns
    /// A [`PoolDevicePath`] instance containing the newly created device path,
    /// or an error if memory could not be allocated.
    pub fn append_path(
        &self,
        path0: &DevicePath,
        path1: &DevicePath,
    ) -> crate::Result<PoolDevicePath> {
        unsafe {
            let ptr =
                (self.0.append_device_path)(path0.as_ffi_ptr().cast(), path1.as_ffi_ptr().cast());
            NonNull::new(ptr.cast_mut())
                .map(|p| PoolDevicePath(PoolAllocation::new(p.cast())))
                .ok_or(Status::OUT_OF_RESOURCES.into())
        }
    }

    /// Creates a new device path by appending a device node to the base device path.
    ///
    /// # Parameters
    /// - `basepath`: A reference to the base device path.
    /// - `node`: A reference to the device node to append.
    ///
    /// # Returns
    /// A [`PoolDevicePath`] instance containing the newly created device path,
    /// or an error if memory could not be allocated.
    pub fn append_node(
        &self,
        basepath: &DevicePath,
        node: &DevicePathNode,
    ) -> crate::Result<PoolDevicePath> {
        unsafe {
            let ptr =
                (self.0.append_device_node)(basepath.as_ffi_ptr().cast(), node.as_ffi_ptr().cast());
            NonNull::new(ptr.cast_mut())
                .map(|p| PoolDevicePath(PoolAllocation::new(p.cast())))
                .ok_or(Status::OUT_OF_RESOURCES.into())
        }
    }

    /// Creates a new device path by appending the specified device path instance to the base path.
    ///
    /// # Parameters
    /// - `basepath`: A reference to the base device path.
    /// - `instance`: A reference to the device path instance to append.
    ///
    /// # Returns
    /// A [`PoolDevicePath`] instance containing the newly created device path,
    /// or an error if memory could not be allocated.
    pub fn append_instance(
        &self,
        basepath: &DevicePath,
        instance: &DevicePath,
    ) -> crate::Result<PoolDevicePath> {
        unsafe {
            let ptr = (self.0.append_device_path_instance)(
                basepath.as_ffi_ptr().cast(),
                instance.as_ffi_ptr().cast(),
            );
            NonNull::new(ptr.cast_mut())
                .map(|p| PoolDevicePath(PoolAllocation::new(p.cast())))
                .ok_or(Status::OUT_OF_RESOURCES.into())
        }
    }
}
