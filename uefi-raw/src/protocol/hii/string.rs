// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bindings for HII String protocols and data types

use super::font::FontInfo;
use super::{HiiHandle, StringId};
use crate::{Char8, Char16, Guid, Status, guid};

/// EFI_HII_STRING_PROTOCOL
#[derive(Debug)]
#[repr(C)]
pub struct HiiStringProtocol {
    pub new_string: unsafe extern "efiapi" fn(
        this: *const Self,
        package_list: HiiHandle,
        string_id: *mut StringId,
        language: *const Char8,
        language_name: *const Char16,
        string: *const Char16,
        string_font_info: *const FontInfo,
    ) -> Status,
    pub get_string: unsafe extern "efiapi" fn(
        this: *const Self,
        language: *const Char8,
        package_list: HiiHandle,
        string_id: StringId,
        string: *mut *mut Char16,
        string_size: *mut usize,
        string_font_info: *mut *mut FontInfo,
    ) -> Status,
    pub set_string: unsafe extern "efiapi" fn(
        this: *const Self,
        package_list: HiiHandle,
        string_id: StringId,
        language: *const Char8,
        string: *const Char16,
        string_font_info: *const FontInfo,
    ) -> Status,
    pub get_languages: unsafe extern "efiapi" fn(
        this: *const Self,
        package_list: HiiHandle,
        languages: *mut Char8,
        languages_size: *mut usize,
    ) -> Status,
    pub get_secondary_languages: unsafe extern "efiapi" fn(
        this: *const Self,
        package_list: HiiHandle,
        primary_language: *const Char8,
        secondary_languages: *mut Char8,
        secondary_languages_size: *mut usize,
    ) -> Status,
}

impl HiiStringProtocol {
    pub const GUID: Guid = guid!("0fd96974-23aa-4cdc-b9cb-98d17750322a");
}
