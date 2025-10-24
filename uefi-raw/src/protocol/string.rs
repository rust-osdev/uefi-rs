// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{Boolean, Char8, Char16, Guid, guid};

#[derive(Debug)]
#[repr(C)]
pub struct UnicodeCollationProtocol {
    pub stri_coll:
        unsafe extern "efiapi" fn(this: *const Self, s1: *const Char16, s2: *const Char16) -> isize,
    pub metai_match: unsafe extern "efiapi" fn(
        this: *const Self,
        string: *const Char16,
        pattern: *const Char16,
    ) -> Boolean,
    pub str_lwr: unsafe extern "efiapi" fn(this: *const Self, s: *mut Char16),
    pub str_upr: unsafe extern "efiapi" fn(this: *const Self, s: *mut Char16),
    pub fat_to_str: unsafe extern "efiapi" fn(
        this: *const Self,
        fat_size: usize,
        fat: *const Char8,
        s: *mut Char16,
    ),
    pub str_to_fat: unsafe extern "efiapi" fn(
        this: *const Self,
        s: *const Char16,
        fat_size: usize,
        fat: *mut Char8,
    ) -> Boolean,
    pub supported_languages: *const Char8,
}

impl UnicodeCollationProtocol {
    pub const GUID: Guid = guid!("a4c751fc-23ae-4c3e-92e9-4964cf63f349");
}
