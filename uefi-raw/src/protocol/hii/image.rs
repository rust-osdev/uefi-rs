// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bindings for HII Image protocols and data types

use super::{HiiHandle, ImageId};
use crate::protocol::console::{GraphicsOutputBltPixel, GraphicsOutputProtocol};
use crate::{Guid, Status, guid};
use core::fmt;

bitflags::bitflags! {
    /// EFI_HII_DRAW_FLAGS
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[repr(transparent)]
    pub struct HiiDrawFlags: u32 {
        const CLIP = 1 << 0;
        const FORCE_TRANS = 1 << 4;
        const FORCE_OPAQUE = 1 << 5;
        const TRANSPARENT = Self::FORCE_TRANS.bits() | Self::FORCE_OPAQUE.bits();
        const DIRECT_TO_SCREEN = 1 << 7;
    }
}

impl HiiDrawFlags {
    pub const DEFAULT: Self = Self::empty();
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[repr(transparent)]
    pub struct ImageInputFlags: u32 {
        const TRANSPARENT = 1 << 0;
    }
}

/// EFI_IMAGE_INPUT
#[derive(Debug)]
#[repr(C)]
pub struct ImageInput {
    pub flags: ImageInputFlags,
    pub width: u16,
    pub height: u16,
    pub bitmap: *const GraphicsOutputBltPixel,
}

#[repr(C)]
pub union ImageOutputDest {
    pub bitmap: *mut GraphicsOutputBltPixel,
    pub screen: *mut GraphicsOutputProtocol,
}

impl fmt::Debug for ImageOutputDest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // All union fields are pointers.
        f.debug_struct("ImageOutputDest")
            .field("bitmap", unsafe { &self.bitmap })
            .field("screen", unsafe { &self.screen })
            .finish()
    }
}

impl Default for ImageOutputDest {
    fn default() -> Self {
        Self {
            bitmap: core::ptr::null_mut(),
        }
    }
}

/// EFI_IMAGE_OUTPUT
#[derive(Debug)]
#[repr(C)]
pub struct ImageOutput {
    pub width: u16,
    pub height: u16,
    pub image: ImageOutputDest,
}

/// EFI_HII_IMAGE_PROTOCOL
#[derive(Debug)]
#[repr(C)]
pub struct HiiImageProtocol {
    pub new_image: unsafe extern "efiapi" fn(
        this: *const Self,
        package_list: HiiHandle,
        image_id: *mut ImageId,
        image: *const ImageInput,
    ) -> Status,
    pub get_image: unsafe extern "efiapi" fn(
        this: *const Self,
        package_list: HiiHandle,
        image_id: ImageId,
        image: *mut ImageInput,
    ) -> Status,
    pub set_image: unsafe extern "efiapi" fn(
        this: *const Self,
        package_list: HiiHandle,
        image_id: ImageId,
        image: *const ImageInput,
    ) -> Status,
    pub draw_image: unsafe extern "efiapi" fn(
        this: *const Self,
        flags: HiiDrawFlags,
        image: *const ImageInput,
        blt: *mut *mut ImageOutput,
        blt_x: usize,
        blt_y: usize,
    ) -> Status,
    pub draw_image_id: unsafe extern "efiapi" fn(
        this: *const Self,
        flags: HiiDrawFlags,
        package_list: HiiHandle,
        image_id: ImageId,
        blt: *mut *mut ImageOutput,
        blt_x: usize,
        blt_y: usize,
    ) -> Status,
}

impl HiiImageProtocol {
    pub const GUID: Guid = guid!("31a6406a-6bdf-4e46-b2a2-ebaa89c40920");
}

/// EFI_HII_IMAGE_EX_PROTOCOL
#[derive(Debug)]
#[repr(C)]
pub struct HiiImageExProtocol {
    // NOTE: UEFI 2.11 declares `image` as an inout value; edk2 declares it as
    // an input-only value, matching the non-extended protocol version.
    pub new_image_ex: unsafe extern "efiapi" fn(
        this: *const Self,
        package_list: HiiHandle,
        image_id: *mut ImageId,
        image: *const ImageInput,
    ) -> Status,
    pub get_image_ex: unsafe extern "efiapi" fn(
        this: *const Self,
        package_list: HiiHandle,
        image_id: ImageId,
        image: *mut ImageInput,
    ) -> Status,
    pub set_image_ex: unsafe extern "efiapi" fn(
        this: *const Self,
        package_list: HiiHandle,
        image_id: ImageId,
        image: *const ImageInput,
    ) -> Status,
    pub draw_image_ex: unsafe extern "efiapi" fn(
        this: *const Self,
        flags: HiiDrawFlags,
        image: *const ImageInput,
        blt: *mut *mut ImageOutput,
        blt_x: usize,
        blt_y: usize,
    ) -> Status,
    pub draw_image_id_ex: unsafe extern "efiapi" fn(
        this: *const Self,
        flags: HiiDrawFlags,
        package_list: HiiHandle,
        image_id: ImageId,
        blt: *mut *mut ImageOutput,
        blt_x: usize,
        blt_y: usize,
    ) -> Status,
    pub get_image_info: unsafe extern "efiapi" fn(
        this: *const Self,
        package_list: HiiHandle,
        image_id: ImageId,
        image: *mut ImageOutput,
    ) -> Status,
}

impl HiiImageExProtocol {
    pub const GUID: Guid = guid!("1a1241e6-8f19-41a9-bc0e-e8ef39e06546");
}
