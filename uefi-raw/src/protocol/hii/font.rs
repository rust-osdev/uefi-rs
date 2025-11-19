// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bindings for HII Font protocols and data types

use super::image::ImageOutput;
use super::{HiiHandle, StringId};
use crate::protocol::console::GraphicsOutputBltPixel;
use crate::{Char8, Char16, Guid, Status, guid};

pub type FontHandle = *mut core::ffi::c_void;

#[derive(Debug)]
#[repr(C)]
pub struct HiiGlyphInfo {
    pub width: u16,
    pub height: u16,
    pub offset_x: i16,
    pub offset_y: i16,
    pub advance_x: i16,
}

bitflags::bitflags! {
    /// EFI_FONT_INFO_MASK
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[repr(transparent)]
    pub struct FontInfoMask: u32 {
        const SYS_FONT = 1 << 0;
        const SYS_SIZE = 1 << 1;
        const SYS_STYLE = 1 << 2;
        const SYS_FORE_COLOR = 1 << 4;
        const SYS_BACK_COLOR = 1 << 5;
        const RESIZE = 1 << 12;
        const RESTYLE = 1 << 13;
        const ANY_FONT = 1 << 16;
        const ANY_SIZE = 1 << 17;
        const ANY_STYLE = 1 << 18;
    }
}

bitflags::bitflags! {
    /// EFI_HII_FONT_STYLE
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[repr(transparent)]
    pub struct HiiFontStyle: u32 {
        const BOLD = 1 << 0;
        const ITALIC = 1 << 1;
        const EMBOSS = 1 << 16;
        const OUTLINE = 1 << 17;
        const SHADOW = 1 << 18;
        const UNDERLINE = 1 << 19;
        const DBL_UNDER = 1 << 20;
    }
}

impl HiiFontStyle {
    pub const NORMAL: Self = Self::empty();
}

/// EFI_FONT_INFO
#[derive(Debug)]
#[repr(C)]
pub struct FontInfo {
    pub font_style: HiiFontStyle,
    pub font_size: u16,
    pub font_name: [Char16; 0],
}

/// EFI_FONT_DISPLAY_INFO
#[derive(Debug)]
#[repr(C)]
pub struct FontDisplayInfo {
    pub foreground_color: GraphicsOutputBltPixel,
    pub background_color: GraphicsOutputBltPixel,
    pub font_mask_info: FontInfoMask,
    pub font_info: FontInfo,
}

bitflags::bitflags! {
    /// EFI_HII_OUT_FLAGS
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[repr(transparent)]
    pub struct HiiOutFlags: u32 {
        const CLIP = 1 << 0;
        const WRAP = 1 << 1;
        const CLIP_CLEAN_Y = 1 << 2;
        const CLIP_CLEAN_X = 1 << 3;
        const TRANSPARENT = 1 << 4;
        const IGNORE_IF_NO_GLYPH = 1 << 5;
        const IGNORE_LINE_BREAK = 1 << 6;
        const DIRECT_TO_SCREEN = 1 << 7;
    }
}

/// EFI_HII_ROW_INFO
#[derive(Debug)]
#[repr(C)]
pub struct HiiRowInfo {
    pub start_index: usize,
    pub end_index: usize,
    pub line_height: usize,
    pub line_width: usize,
    pub baseline_offset: usize,
}

/// EFI_HII_FONT_PROTOCOL
#[derive(Debug)]
#[repr(C)]
pub struct HiiFontProtocol {
    pub string_to_image: unsafe extern "efiapi" fn(
        this: *const Self,
        flags: HiiOutFlags,
        string: *const Char16,
        string_info: *const FontDisplayInfo,
        blt: *mut *mut ImageOutput,
        blt_x: usize,
        blt_y: usize,
        row_info_array: *mut *mut HiiRowInfo,
        row_info_array_size: *mut usize,
        column_info_array: *mut usize,
    ) -> Status,
    pub string_id_to_image: unsafe extern "efiapi" fn(
        this: *const Self,
        flags: HiiOutFlags,
        package_list: HiiHandle,
        string_id: StringId,
        language: *const Char8,
        string_info: *const FontDisplayInfo,
        blt: *mut *mut ImageOutput,
        blt_x: usize,
        blt_y: usize,
        row_info_array: *mut *mut HiiRowInfo,
        row_info_array_size: *mut usize,
        column_info_array: *mut usize,
    ) -> Status,
    pub get_glyph: unsafe extern "efiapi" fn(
        this: *const Self,
        char: Char16,
        string_info: *const FontDisplayInfo,
        blt: *mut *mut ImageOutput,
        baseline: *mut usize,
    ) -> Status,
    pub get_font_info: unsafe extern "efiapi" fn(
        this: *const Self,
        font_handle: *mut FontHandle,
        string_info_in: *const FontDisplayInfo,
        string_info_out: *mut *mut FontDisplayInfo,
        string: *const Char16,
    ) -> Status,
}

impl HiiFontProtocol {
    pub const GUID: Guid = guid!("e9ca4775-8657-47fc-97e7-7ed65a084324");
}

// NOTE: This protocol is declared in the UEFI spec, but not defined in edk2.
/// EFI_HII_FONT_EX_PROTOCOL
#[derive(Debug)]
#[repr(C)]
pub struct HiiFontExProtocol {
    pub string_to_image_ex: unsafe extern "efiapi" fn(
        this: *const Self,
        flags: HiiOutFlags,
        string: *const Char16,
        string_info: *const FontDisplayInfo,
        blt: *mut *mut ImageOutput,
        blt_x: usize,
        blt_y: usize,
        row_info_array: *mut *mut HiiRowInfo,
        row_info_array_size: *mut usize,
        column_info_array: *mut usize,
    ) -> Status,
    pub string_id_to_image_ex: unsafe extern "efiapi" fn(
        this: *const Self,
        flags: HiiOutFlags,
        package_list: HiiHandle,
        string_id: StringId,
        language: *const Char8,
        string_info: *const FontDisplayInfo,
        blt: *mut *mut ImageOutput,
        blt_x: usize,
        blt_y: usize,
        row_info_array: *mut *mut HiiRowInfo,
        row_info_array_size: *mut usize,
        column_info_array: *mut usize,
    ) -> Status,
    pub get_glyph_ex: unsafe extern "efiapi" fn(
        this: *const Self,
        char: Char16,
        string_info: *const FontDisplayInfo,
        blt: *mut *mut ImageOutput,
        baseline: *mut usize,
    ) -> Status,
    pub get_font_info_ex: unsafe extern "efiapi" fn(
        this: *const Self,
        font_handle: *mut FontHandle,
        string_info_in: *const FontDisplayInfo,
        string_info_out: *mut *mut FontDisplayInfo,
        string: *const Char16,
    ) -> Status,
    pub get_glyph_info: unsafe extern "efiapi" fn(
        this: *const Self,
        char: Char16,
        font_display_info: *const FontDisplayInfo,
        glyph_info: *mut HiiGlyphInfo,
    ) -> Status,
}

impl HiiFontExProtocol {
    pub const GUID: Guid = guid!("849e6875-db35-4df8-b41e-c8f33718073f");
}
