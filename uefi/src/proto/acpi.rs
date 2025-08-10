// SPDX-License-Identifier: MIT OR Apache-2.0

//! `AcpiTable` protocol.

use crate::proto::unsafe_protocol;
use crate::{Result, StatusExt};
use core::ffi::c_void;
use uefi_raw::protocol::acpi::AcpiTableProtocol;

/// The AcpiTable protocol.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(AcpiTableProtocol::GUID)]
pub struct AcpiTable(AcpiTableProtocol);

impl AcpiTable {
    /// Installs an ACPI table into the RSDT/XSDT. Returns a index
    /// that may be used by `uninstall_acpi_table` to remove the ACPI
    /// table.
    ///
    /// # Safety
    ///
    /// When installing ACPI table, the data pointed to by
    /// `acpi_table_ptr` must be a pool allocation of type
    /// [`ACPI_RECLAIM`] or other type suitable for data handed off to
    /// the OS.
    ///
    /// [`ACPI_RECLAIM`]: crate::boot::MemoryType::ACPI_RECLAIM
    ///
    /// # Errors
    ///
    /// * [`Status::INVALID_PARAMETER`]: `acpi_table_ptr` is null; the
    ///   `acpi_table_size`, and the size field embedded in the ACPI
    ///   table are not in sync.
    ///
    /// * [`Status::OUT_OF_RESOURCES`]: Insufficient resources
    ///   exist to complete the request.
    ///
    /// * [`Status::ACCESS_DENIED`]: The table signature matches a
    ///   table already present in the system and platform policy does
    ///   not allow duplicate tables of this type.
    ///
    /// [`Status::INVALID_PARAMETER`]: crate::Status::INVALID_PARAMETER
    /// [`Status::OUT_OF_RESOURCES`]: crate::Status::OUT_OF_RESOURCES
    /// [`Status::ACCESS_DENIED`]: crate::Status::ACCESS_DENIED
    pub unsafe fn install_acpi_table(
        &self,
        acpi_table_ptr: *const c_void,
        acpi_table_size: usize,
    ) -> Result<usize> {
        let mut table_key = 0usize;
        let status = unsafe {
            (self.0.install_acpi_table)(&self.0, acpi_table_ptr, acpi_table_size, &mut table_key)
        };
        status.to_result_with_val(|| table_key)
    }

    /// Removes an ACPI table from the RSDT/XSDT.
    ///
    /// # Errors
    ///
    /// * [`Status::NOT_FOUND`]: `table_key` does not refer to a
    ///   valid key for a table entry.
    ///
    /// * [`Status::OUT_OF_RESOURCES`]: Insufficient resources exist
    ///   to complete the request.
    ///
    /// [`Status::NOT_FOUND`]: crate::Status::NOT_FOUND
    /// [`Status::OUT_OF_RESOURCES`]: crate::Status::OUT_OF_RESOURCES
    pub fn uninstall_acpi_table(&self, table_key: usize) -> Result {
        unsafe { (self.0.uninstall_acpi_table)(&self.0, table_key) }.to_result()
    }
}
