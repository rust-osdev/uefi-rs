//! Load file support protocols.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::proto::device_path::{FfiDevicePath, DevicePath};
use crate::proto::unsafe_protocol;
use crate::{Result, Status};
use core::ffi::c_void;
use core::ptr;

/// The UEFI LoadFile2 protocol.
///
/// This protocol has a single method to load a file according to some
/// device path.
///
/// This interface is implemented by many devices, e.g. network and filesystems.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(uefi_raw::protocol::media::LoadFile2::GUID)]
pub struct LoadFile2(uefi_raw::protocol::media::LoadFile2);

impl LoadFile2 {
    /// Load file addressed by provided device path
    pub fn load_file(&mut self,
        file_path: &DevicePath,
        buffer: &mut [u8]
    ) -> Result<(), usize> {
        let mut buffer_size = buffer.len();
        unsafe {
            (self.0.load_file)(self,
                file_path,
                false,
                buffer_size,
                buffer.as_mut_ptr()
            )
        }.to_result_with(
            || debug_assert_eq!(buffer_size, buffer.len()),
            |_| buffer_size
        )
    }

    #[cfg(feature = "alloc")]
    /// Load file addressed by the provided device path.
    pub fn load_file_to_vec(&mut self,
        file_path: &DevicePath,
    ) -> Result<Vec<u8>> {
        let mut buffer_size: usize = 0;
        let mut status: Status;
        unsafe {
            status = (self.0.load_file)(self,
                file_path.as_ffi_ptr(),
                false,
                ptr::addr_of_mut!(buffer_size),
                ptr::null_mut()
            );
        }

        if status.is_error() {
            return Err(status.into());
        }

        let mut buffer: Vec<u8> = Vec::with_capacity(buffer_size);
        unsafe {
            status = (self.0.load_file)(self,
                file_path.as_ffi_ptr(),
                false,
                ptr::addr_of_mut!(buffer_size),
                buffer.as_mut_ptr() as *mut c_void
            );
        }

        if status.is_error() {
            return Err(status.into());
        }

        Ok(buffer)
    }
}
