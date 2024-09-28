#![allow(deprecated)]

use super::{cfg, Revision};
use crate::proto::console::text;
use crate::CStr16;
use core::ffi::c_void;
use core::marker::PhantomData;
use core::ptr::NonNull;
use core::slice;

/// Marker trait used to provide different views of the UEFI System Table.
#[deprecated = "Use the uefi::system, uefi::boot, and uefi::runtime modules instead. See https://github.com/rust-osdev/uefi-rs/blob/HEAD/docs/funcs_migration.md"]
pub trait SystemTableView {}

/// Marker struct associated with the boot view of the UEFI System Table.
#[deprecated = "Use the uefi::boot module instead. See https://github.com/rust-osdev/uefi-rs/blob/HEAD/docs/funcs_migration.md"]
#[derive(Debug)]
pub struct Boot;
impl SystemTableView for Boot {}

/// UEFI System Table interface
///
/// The UEFI System Table is the gateway to all UEFI services which an UEFI
/// application is provided access to on startup. However, not all UEFI services
/// will remain accessible forever.
///
/// Some services, called "boot services", may only be called during a bootstrap
/// stage where the UEFI firmware still has control of the hardware, and will
/// become unavailable once the firmware hands over control of the hardware to
/// an operating system loader. Others, called "runtime services", may still be
/// used after that point, but require a rather specific CPU configuration which
/// an operating system loader is unlikely to preserve.
///
/// We handle this state transition by providing two different views of the UEFI
/// system table, the "Boot" view and the "Runtime" view. An UEFI application
/// is initially provided with access to the "Boot" view, and may transition
/// to the "Runtime" view through the ExitBootServices mechanism that is
/// documented in the UEFI spec. At that point, the boot view of the system
/// table will be destroyed (which conveniently invalidates all references to
/// UEFI boot services in the eye of the Rust borrow checker) and a runtime view
/// will be provided to replace it
#[deprecated = "Use the uefi::system, uefi::boot, and uefi::runtime modules instead. See https://github.com/rust-osdev/uefi-rs/blob/HEAD/docs/funcs_migration.md"]
#[derive(Debug)]
#[repr(transparent)]
pub struct SystemTable<View: SystemTableView> {
    table: *const uefi_raw::table::system::SystemTable,
    _marker: PhantomData<View>,
}

// These parts of the UEFI System Table interface will always be available
impl<View: SystemTableView> SystemTable<View> {
    /// Return the firmware vendor string
    #[must_use]
    pub fn firmware_vendor(&self) -> &CStr16 {
        unsafe { CStr16::from_ptr((*self.table).firmware_vendor.cast()) }
    }

    /// Return the firmware revision
    #[must_use]
    pub const fn firmware_revision(&self) -> u32 {
        unsafe { (*self.table).firmware_revision }
    }

    /// Returns the revision of this table, which is defined to be
    /// the revision of the UEFI specification implemented by the firmware.
    #[must_use]
    pub const fn uefi_revision(&self) -> Revision {
        unsafe { (*self.table).header.revision }
    }

    /// Returns the config table entries, a linear array of structures
    /// pointing to other system-specific tables.
    #[allow(clippy::missing_const_for_fn)] // Required until we bump the MSRV.
    #[must_use]
    pub fn config_table(&self) -> &[cfg::ConfigTableEntry] {
        unsafe {
            let table = &*self.table;
            table
                .configuration_table
                .cast::<cfg::ConfigTableEntry>()
                .as_ref()
                .map(|ptr| slice::from_raw_parts(ptr, table.number_of_configuration_table_entries))
                .unwrap_or(&[])
        }
    }

    /// Creates a new `SystemTable<View>` from a raw address. The address might
    /// come from the Multiboot2 information structure or something similar.
    ///
    /// # Example
    /// ```no_run
    /// use core::ffi::c_void;
    /// use uefi::prelude::{Boot, SystemTable};
    ///
    /// let system_table_addr = 0xdeadbeef as *mut c_void;
    ///
    /// let mut uefi_system_table = unsafe {
    ///     SystemTable::<Boot>::from_ptr(system_table_addr).expect("Pointer must not be null!")
    /// };
    /// ```
    ///
    /// # Safety
    /// This function is unsafe because the caller must be sure that the pointer
    /// is valid. Otherwise, further operations on the object might result in
    /// undefined behaviour, even if the methods aren't marked as unsafe.
    pub unsafe fn from_ptr(ptr: *mut c_void) -> Option<Self> {
        NonNull::new(ptr.cast()).map(|ptr| Self {
            table: ptr.as_ref(),
            _marker: PhantomData,
        })
    }

    /// Get the underlying raw pointer.
    #[must_use]
    pub const fn as_ptr(&self) -> *const c_void {
        self.table.cast()
    }
}

// These parts of the UEFI System Table interface may only be used until boot
// services are exited and hardware control is handed over to the OS loader
impl SystemTable<Boot> {
    /// Returns the standard input protocol.
    pub fn stdin(&mut self) -> &mut text::Input {
        unsafe { &mut *(*self.table).stdin.cast() }
    }

    /// Returns the standard output protocol.
    pub fn stdout(&mut self) -> &mut text::Output {
        unsafe { &mut *(*self.table).stdout.cast() }
    }

    /// Returns the standard error protocol.
    pub fn stderr(&mut self) -> &mut text::Output {
        unsafe { &mut *(*self.table).stderr.cast() }
    }

    /// Clone this boot-time UEFI system table interface
    ///
    /// # Safety
    ///
    /// This is unsafe because you must guarantee that the clone will not be
    /// used after boot services are exited. However, the singleton-based
    /// designs that Rust uses for memory allocation, logging, and panic
    /// handling require taking this risk.
    #[must_use]
    pub const unsafe fn unsafe_clone(&self) -> Self {
        Self {
            table: self.table,
            _marker: PhantomData,
        }
    }
}

impl<View: SystemTableView> super::Table for SystemTable<View> {
    const SIGNATURE: u64 = 0x5453_5953_2049_4249;
}
