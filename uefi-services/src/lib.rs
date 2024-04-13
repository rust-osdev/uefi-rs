//! WARNING: `uefi-services` is deprecated. Functionality was moved to `uefi::helpers::init`.
#![no_std]

use uefi::prelude::*;
use uefi::Event;
use uefi::Result;

pub use uefi::{print, println};

/// Deprecated. Use [`uefi::helpers::init`] instead.
#[deprecated = "WARNING: `uefi-services` is deprecated. Functionality was moved to `uefi::helpers::init`."]
pub fn init(st: &mut SystemTable<Boot>) -> Result<Option<Event>> {
    uefi::helpers::init(st)
}

/// Deprecated. Use [`uefi::helpers::system_table`] instead.
#[deprecated = "WARNING: `uefi-services` is deprecated. Functionality was moved to `uefi::helpers::system_table`."]
pub fn system_table() -> SystemTable<Boot> {
    uefi::helpers::system_table()
}
