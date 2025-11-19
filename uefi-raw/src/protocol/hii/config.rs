// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bindings for HII protocols relating to system configuration.

use core::fmt::Debug;

use super::form_browser::BrowserActionRequest;
use super::{FormId, QuestionId, StringId};
use crate::protocol::device_path::DevicePathProtocol;
use crate::{Boolean, Char16, Guid, Status, guid, newtype_enum};

/// EFI_CONFIG_KEYWORD_HANDLER_PROTOCOL
#[derive(Debug)]
#[repr(C)]
pub struct ConfigKeywordHandlerProtocol {
    pub set_data: unsafe extern "efiapi" fn(
        this: *mut Self,
        keyword_string: *const Char16,
        progress: *mut *const Char16,
        progress_err: *mut u32,
    ) -> Status,
    pub get_data: unsafe extern "efiapi" fn(
        this: *const Self,
        namespace_id: *const Char16,
        keyword_string: *const Char16,
        progress: *mut *const Char16,
        progress_err: *mut u32,
        results: *mut *const Char16,
    ) -> Status,
}

impl ConfigKeywordHandlerProtocol {
    pub const GUID: Guid = guid!("0a8badd5-03b8-4d19-b128-7b8f0edaa596");
}

newtype_enum! {
    /// Type of action taken by the form browser
    #[derive(Default)]
    pub enum BrowserAction: usize => {
        /// Called before the browser changes the value in the display (for questions which have a value)
        /// or takes an action (in the case of an action button or cross-reference).
        /// If EFI_SUCCESS is returned, the browser uses the value returned by Callback().
        CHANGING = 0,
        /// Called after the browser has changed its internal copy of the question value and displayed it (if appropriate).
        /// For action buttons, this is called after processing. Errors are ignored.
        CHANGED = 1,
        /// Called after the browser has read the current question value but before displaying it.
        /// If EFI_SUCCESS is returned, the updated value is used.
        RETRIEVE = 2,
        /// Called for each question on a form prior to its value being retrieved or displayed.
        /// If a question appears on more than one form, this may be called more than once.
        FORM_OPEN = 3,
        /// Called for each question on a form after processing any submit actions for that form.
        /// If a question appears on multiple forms, this will be called more than once.
        FORM_CLOSE = 4,
        /// Called after the browser submits the modified question value.
        /// ActionRequest is ignored.
        SUBMITTED = 5,
        /// Represents the standard default action, selecting a default value based on lower-priority methods.
        DEFAULT_STANDARD = 0x1000,
        /// Represents the manufacturing default action, selecting a default value relevant to manufacturing.
        DEFAULT_MANUFACTURING = 0x1001,
        /// Represents the safe default action, selecting the safest possible default value.
        DEFAULT_SAFE = 0x1002,
        /// Represents platform-defined default values within a range of possible store identifiers.
        DEFAULT_PLATFORM = 0x2000,
        /// Represents hardware-defined default values within a range of possible store identifiers.
        DEFAULT_HARDWARE = 0x3000,
        /// Represents firmware-defined default values within a range of possible store identifiers.
        DEFAULT_FIRMWARE = 0x4000,
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct HiiTime {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct HiiDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct HiiRef {
    pub question_id: QuestionId,
    pub form_id: FormId,
    pub guid: Guid,
    pub string_id: StringId,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union IfrTypeValue {
    pub u8: u8,           // EFI_IFR_TYPE_NUM_SIZE_8
    pub u16: u16,         // EFI_IFR_TYPE_NUM_SIZE_16
    pub u32: u32,         // EFI_IFR_TYPE_NUM_SIZE_32
    pub u64: u64,         // EFI_IFR_TYPE_NUM_SIZE_64
    pub b: Boolean,       // EFI_IFR_TYPE_BOOLEAN
    pub time: HiiTime,    // EFI_IFR_TYPE_TIME
    pub date: HiiDate,    // EFI_IFR_TYPE_DATE
    pub string: StringId, // EFI_IFR_TYPE_STRING, EFI_IFR_TYPE_ACTION
    pub hii_ref: HiiRef,  // EFI_IFR_TYPE_REF
}
impl core::fmt::Debug for IfrTypeValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EfiIfrTypeValue").finish()
    }
}

/// EFI_HII_CONFIG_ACCESS_PROTOCOL
#[derive(Debug)]
#[repr(C)]
pub struct HiiConfigAccessProtocol {
    pub extract_config: unsafe extern "efiapi" fn(
        this: *const Self,
        request: *const Char16,
        progress: *mut *const Char16,
        results: *mut *const Char16,
    ) -> Status,
    pub route_config: unsafe extern "efiapi" fn(
        this: *const Self,
        configuration: *const Char16,
        progress: *mut *const Char16,
    ) -> Status,
    pub callback: unsafe extern "efiapi" fn(
        this: *const Self,
        action: BrowserAction,
        question_id: QuestionId,
        value_type: u8,
        value: *mut IfrTypeValue,
        action_request: *mut BrowserActionRequest,
    ) -> Status,
}

impl HiiConfigAccessProtocol {
    pub const GUID: Guid = guid!("330d4706-f2a0-4e4f-a369-b66fa8d54385");
}

/// EFI_HII_CONFIG_ROUTING_PROTOCOL
#[derive(Debug)]
#[repr(C)]
pub struct HiiConfigRoutingProtocol {
    pub extract_config: unsafe extern "efiapi" fn(
        this: *const Self,
        config_request: *const Char16,
        progress: *mut *const Char16,
        results: *mut *const Char16,
    ) -> Status,
    pub export_config:
        unsafe extern "efiapi" fn(this: *const Self, results: *mut *const Char16) -> Status,
    pub route_config: unsafe extern "efiapi" fn(
        this: *const Self,
        configuration: *const Char16,
        progress: *mut *const Char16,
    ) -> Status,
    pub block_to_config: unsafe extern "efiapi" fn(
        this: *const Self,
        config_request: *const Char16,
        block: *const u8,
        block_size: usize,
        config: *mut *const Char16,
        progress: *mut *const Char16,
    ) -> Status,
    pub config_to_block: unsafe extern "efiapi" fn(
        this: *const Self,
        config_resp: *const Char16,
        block: *mut *const u8,
        block_size: *mut usize,
        progress: *mut *const Char16,
    ) -> Status,
    pub get_alt_cfg: unsafe extern "efiapi" fn(
        this: *const Self,
        config_resp: *const Char16,
        guid: *const Guid,
        name: *const Char16,
        device_path: *const DevicePathProtocol,
        alt_cfg_id: *const Char16,
        alt_cfg_resp: *mut *const Char16,
    ) -> Status,
}

impl HiiConfigRoutingProtocol {
    pub const GUID: Guid = guid!("587e72d7-cc50-4f79-8209-ca291fc1a10f");
}
