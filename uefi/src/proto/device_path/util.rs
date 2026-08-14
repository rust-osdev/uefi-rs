// SPDX-License-Identifier: MIT OR Apache-2.0

//! Protocol with utility functions for working with device paths.

use super::{DevicePath, DevicePathNode, PoolDevicePath};
use crate::mem::PoolAllocation;
use core::ptr::NonNull;
use uefi_macros::unsafe_protocol;
use uefi_raw::Status;
use uefi_raw::protocol::device_path::DevicePathUtilitiesProtocol;

/// Protocol with utility functions for working with device paths.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(DevicePathUtilitiesProtocol::GUID)]
pub struct DevicePathUtilities(DevicePathUtilitiesProtocol);

impl DevicePathUtilities {
    /// Returns the size of `device_path` in bytes, including its
    /// end-of-device-path node.
    #[must_use]
    pub fn get_size(&self, device_path: &DevicePath) -> usize {
        // SAFETY: The memory is valid.
        unsafe { (self.0.get_device_path_size)(device_path.as_ffi_ptr().cast()) }
    }

    /// Clones `path` into a newly allocated [`PoolDevicePath`].
    ///
    /// # Errors
    ///
    /// Returns [`Status::OUT_OF_RESOURCES`] if allocation fails.
    pub fn duplicate_path(&self, path: &DevicePath) -> crate::Result<PoolDevicePath> {
        // SAFETY: The memory is valid.
        unsafe {
            let ptr = (self.0.duplicate_device_path)(path.as_ffi_ptr().cast());
            NonNull::new(ptr.cast_mut())
                .map(|p| PoolDevicePath(PoolAllocation::new(p.cast())))
                .ok_or_else(|| Status::OUT_OF_RESOURCES.into())
        }
    }

    /// Appends `path1` to `path0` in a newly allocated [`PoolDevicePath`].
    ///
    /// # Errors
    ///
    /// Returns [`Status::OUT_OF_RESOURCES`] if allocation fails.
    pub fn append_path(
        &self,
        path0: &DevicePath,
        path1: &DevicePath,
    ) -> crate::Result<PoolDevicePath> {
        // SAFETY: The memory is valid.
        unsafe {
            let ptr =
                (self.0.append_device_path)(path0.as_ffi_ptr().cast(), path1.as_ffi_ptr().cast());
            NonNull::new(ptr.cast_mut())
                .map(|p| PoolDevicePath(PoolAllocation::new(p.cast())))
                .ok_or_else(|| Status::OUT_OF_RESOURCES.into())
        }
    }

    /// Appends `node` to `basepath` in a newly allocated [`PoolDevicePath`].
    ///
    /// # Errors
    ///
    /// Returns [`Status::OUT_OF_RESOURCES`] if allocation fails.
    pub fn append_node(
        &self,
        basepath: &DevicePath,
        node: &DevicePathNode,
    ) -> crate::Result<PoolDevicePath> {
        // SAFETY: The memory is valid.
        unsafe {
            let ptr =
                (self.0.append_device_node)(basepath.as_ffi_ptr().cast(), node.as_ffi_ptr().cast());
            NonNull::new(ptr.cast_mut())
                .map(|p| PoolDevicePath(PoolAllocation::new(p.cast())))
                .ok_or_else(|| Status::OUT_OF_RESOURCES.into())
        }
    }

    /// Appends `instance` to `basepath` in a newly allocated [`PoolDevicePath`].
    ///
    /// # Errors
    ///
    /// Returns [`Status::OUT_OF_RESOURCES`] if allocation fails.
    pub fn append_instance(
        &self,
        basepath: &DevicePath,
        instance: &DevicePath,
    ) -> crate::Result<PoolDevicePath> {
        // SAFETY: The memory is valid.
        unsafe {
            let ptr = (self.0.append_device_path_instance)(
                basepath.as_ffi_ptr().cast(),
                instance.as_ffi_ptr().cast(),
            );
            NonNull::new(ptr.cast_mut())
                .map(|p| PoolDevicePath(PoolAllocation::new(p.cast())))
                .ok_or_else(|| Status::OUT_OF_RESOURCES.into())
        }
    }
}
