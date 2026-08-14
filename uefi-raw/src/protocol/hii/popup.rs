// SPDX-License-Identifier: MIT OR Apache-2.0

//! Raw HII popup protocol.

use super::{HiiHandle, StringId};
use crate::{Guid, Status, guid, newtype_enum};

newtype_enum! {
    /// Visual style of an HII popup.
    pub enum HiiPopupStyle: u32 => {
        INFO = 0,
        WARNING = 1,
        ERROR = 2,
    }
}

newtype_enum! {
    /// Buttons displayed by an HII popup.
    pub enum HiiPopupType: u32 => {
        OK = 0,
        OK_CANCEL = 1,
        YES_NO = 2,
        YES_NO_CANCEL = 3,
    }
}

newtype_enum! {
    /// Button selected in an HII popup.
    pub enum HiiPopupSelection: u32 => {
        OK = 0,
        CANCEL = 1,
        YES = 2,
        NO = 3,
    }
}

/// HII popup protocol.
#[derive(Debug)]
#[repr(C)]
pub struct HiiPopupProtocol {
    pub revision: u64,
    pub create_popup: unsafe extern "efiapi" fn(
        this: *const Self,
        popup_style: HiiPopupStyle,
        popup_type: HiiPopupType,
        hii_handle: HiiHandle,
        message: StringId,
        user_selection: *mut HiiPopupSelection,
    ) -> Status,
}

impl HiiPopupProtocol {
    pub const GUID: Guid = guid!("4311edc0-6054-46d4-9e40-893ea952fccc");
    pub const REVISION: u64 = 1;
}
