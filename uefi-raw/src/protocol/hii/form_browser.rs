// SPDX-License-Identifier: MIT OR Apache-2.0

//! Form Browser protocol

use super::{FormId, HiiHandle};
use crate::{Boolean, Char16, Guid, Status, guid, newtype_enum};

/// EFI_SCREEN_DESCRIPTOR
#[derive(Debug)]
#[repr(C)]
pub struct ScreenDescriptor {
    pub left_column: usize,
    pub right_column: usize,
    pub top_row: usize,
    pub bottom_row: usize,
}

newtype_enum! {
    /// Represents actions requested by the Forms Browser in response to user interactions.
    #[derive(Default)]
    pub enum BrowserActionRequest: usize => {
        /// No special behavior is taken by the Forms Browser.
        NONE = 0,
        /// The Forms Browser will exit and request the platform to reset.
        RESET = 1,
        /// The Forms Browser will save all modified question values to storage and exit.
        SUBMIT = 2,
        /// The Forms Browser will discard all modified question values and exit.
        EXIT = 3,
        /// The Forms Browser will write all modified question values on the selected form to storage and exit the form.
        FORM_SUBMIT_EXIT = 4,
        /// The Forms Browser will discard the modified question values on the selected form and exit the form.
        FORM_DISCARD_EXIT = 5,
        /// The Forms Browser will write all modified current question values on the selected form to storage.
        FORM_APPLY = 6,
        /// The Forms Browser will discard the current question values on the selected form and replace them with the original values.
        FORM_DISCARD = 7,
        /// The user performed a hardware or software configuration change, requiring controller reconnection.
        /// The Forms Browser calls `DisconnectController()` followed by `ConnectController()`.
        RECONNECT = 8,
        /// The Forms Browser will write the current modified question value on the selected form to storage.
        QUESTION_APPLY = 9,
    }
}

/// EFI_FORM_BROWSER2_PROTOCOL
#[derive(Debug)]
#[repr(C)]
pub struct FormBrowser2Protocol {
    pub send_form: unsafe extern "efiapi" fn(
        this: *const Self,
        handles: *const HiiHandle,
        handle_count: usize,
        formset_guid: *const Guid,
        form_id: FormId,
        screen_dimensions: *const ScreenDescriptor,
        action_request: *mut BrowserActionRequest,
    ) -> Status,
    pub browser_callback: unsafe extern "efiapi" fn(
        this: *const Self,
        results_data_size: *mut usize,
        results_data: *mut Char16,
        retrieve_data: Boolean,
        variable_guid: *const Guid,
        variable_name: *const Char16,
    ) -> Status,
}

impl FormBrowser2Protocol {
    pub const GUID: Guid = guid!("b9d4c360-bcfb-4f9b-9298-53c136982258");
}
