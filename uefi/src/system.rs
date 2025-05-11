// SPDX-License-Identifier: MIT OR Apache-2.0

//! Functions for accessing fields of the system table.
//!
//! Some of these functions use a callback argument rather than returning a
//! reference to the field directly. This pattern is used because some fields
//! are allowed to change, and so a static lifetime cannot be used.
//!
//! Some functions can only be called while boot services are active, and will
//! panic otherwise. See each function's documentation for details.

use crate::proto::console::text::{Input, Output};
use crate::table::cfg::ConfigTableEntry;
use crate::table::{self, Revision};
use crate::{CStr16, Char16};
use core::slice;

/// Get the firmware vendor string.
#[must_use]
pub fn firmware_vendor() -> &'static CStr16 {
    let st = table::system_table_raw_panicking();
    // SAFETY: valid per requirements of `set_system_table`.
    let st = unsafe { st.as_ref() };

    let vendor: *const Char16 = st.firmware_vendor.cast();

    // SAFETY: this assumes that the firmware vendor string is never mutated or freed.
    unsafe { CStr16::from_ptr(vendor) }
}

/// Get the firmware revision.
#[must_use]
pub fn firmware_revision() -> u32 {
    let st = table::system_table_raw_panicking();
    // SAFETY: valid per requirements of `set_system_table`.
    let st = unsafe { st.as_ref() };

    st.firmware_revision
}

/// Get the revision of the system table, which is defined to be the revision of
/// the UEFI specification implemented by the firmware.
#[must_use]
pub fn uefi_revision() -> Revision {
    let st = table::system_table_raw_panicking();
    // SAFETY: valid per requirements of `set_system_table`.
    let st = unsafe { st.as_ref() };

    st.header.revision
}

/// Call `f` with a slice of [`ConfigTableEntry`]. Each entry provides access to
/// a vendor-specific table.
///
/// # Example
///
/// ```rust,no_run
/// use uefi::system::with_config_table;
/// use uefi::table::cfg::ConfigTableEntry;
///
/// with_config_table(|slice| {
///     for i in slice {
///         match i.guid {
///             ConfigTableEntry::ACPI_GUID => println!("Found ACPI1"),
///             ConfigTableEntry::ACPI2_GUID => println!("Found ACPI2"),
///             guid => println!("Found {}", guid),
///         }
///     }
/// });
/// ```
pub fn with_config_table<F, R>(mut f: F) -> R
where
    F: FnMut(&[ConfigTableEntry]) -> R,
{
    let st = table::system_table_raw_panicking();
    // SAFETY: valid per requirements of `set_system_table`.
    let st = unsafe { st.as_ref() };

    let ptr: *const ConfigTableEntry = st.configuration_table.cast();
    let len = st.number_of_configuration_table_entries;
    let slice = if ptr.is_null() {
        &[]
    } else {
        unsafe { slice::from_raw_parts(ptr, len) }
    };

    f(slice)
}

/// Call `f` with the [`Input`] protocol attached to stdin.
///
/// # Panics
///
/// This function will panic if called after exiting boot services, or if stdin
/// is not available.
pub fn with_stdin<F, R>(mut f: F) -> R
where
    F: FnMut(&mut Input) -> R,
{
    let st = table::system_table_raw_panicking();
    // SAFETY: valid per requirements of `set_system_table`.
    let st = unsafe { st.as_ref() };
    // The I/O protocols cannot be used after exiting boot services.
    assert!(!st.boot_services.is_null(), "boot services are not active");
    assert!(!st.stdin.is_null(), "stdin is not available");

    let stdin: *mut Input = st.stdin.cast();

    // SAFETY: `Input` is a `repr(transparent)` wrapper around the raw input
    // type. The underlying pointer in the system table is assumed to be valid.
    let stdin = unsafe { &mut *stdin };

    f(stdin)
}

/// Call `f` with the [`Output`] protocol attached to stdout.
///
/// # Panics
///
/// This function will panic if called after exiting boot services, or if stdout
/// is not available.
pub fn with_stdout<F, R>(mut f: F) -> R
where
    F: FnMut(&mut Output) -> R,
{
    let st = table::system_table_raw_panicking();
    // SAFETY: valid per requirements of `set_system_table`.
    let st = unsafe { st.as_ref() };
    // The I/O protocols cannot be used after exiting boot services.
    assert!(!st.boot_services.is_null(), "boot services are not active");
    assert!(!st.stdout.is_null(), "stdout is not available");

    let stdout: *mut Output = st.stdout.cast();

    // SAFETY: `Output` is a `repr(transparent)` wrapper around the raw output
    // type. The underlying pointer in the system table is assumed to be valid.
    let stdout = unsafe { &mut *stdout };

    f(stdout)
}

/// Call `f` with the [`Output`] protocol attached to stderr.
///
/// # Panics
///
/// This function will panic if called after exiting boot services, or if stderr
/// is not available.
pub fn with_stderr<F, R>(mut f: F) -> R
where
    F: FnMut(&mut Output) -> R,
{
    let st = table::system_table_raw_panicking();
    // SAFETY: valid per requirements of `set_system_table`.
    let st = unsafe { st.as_ref() };
    // The I/O protocols cannot be used after exiting boot services.
    assert!(!st.boot_services.is_null(), "boot services are not active");
    assert!(!st.stderr.is_null(), "stderr is not available");

    let stderr: *mut Output = st.stderr.cast();

    // SAFETY: `Output` is a `repr(transparent)` wrapper around the raw output
    // type. The underlying pointer in the system table is assumed to be valid.
    let stderr = unsafe { &mut *stderr };

    f(stderr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    #[allow(clippy::assertions_on_constants)]
    fn with_config_table_compile_test() {
        assert!(false, "compile test only");

        let mut acpi2_address = None;

        with_config_table(|slice| {
            for i in slice {
                match i.guid {
                    ConfigTableEntry::ACPI2_GUID => {
                        acpi2_address = Some(i.address);
                        break;
                    }
                    _ => {}
                }
            }
        });
    }
}
