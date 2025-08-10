// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{Guid, Status, guid};
use core::ffi::c_void;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct AcpiTableProtocol {
    pub install_acpi_table: unsafe extern "efiapi" fn(
        this: *const Self,
        acpi_table_buffer: *const c_void,
        acpi_table_size: usize,
        table_key: *mut usize,
    ) -> Status,
    pub uninstall_acpi_table:
        unsafe extern "efiapi" fn(this: *const Self, table_key: usize) -> Status,
}

impl AcpiTableProtocol {
    pub const GUID: Guid = guid!("ffe06bdd-6107-46a6-7bb2-5a9c7ec5275c");
}
