#![allow(deprecated)]

use super::boot::BootServices;
use super::runtime::{ResetType, RuntimeServices};
use super::{cfg, Revision};
use crate::proto::console::text;
use crate::{CStr16, Result, Status};
use core::ffi::c_void;
use core::marker::PhantomData;
use core::ptr::NonNull;
use core::slice;
use uefi::mem::memory_map::{MemoryMapBackingMemory, MemoryMapMeta, MemoryMapOwned, MemoryType};

/// Marker trait used to provide different views of the UEFI System Table.
#[deprecated = "Use the uefi::system, uefi::boot, and uefi::runtime modules instead. See https://github.com/rust-osdev/uefi-rs/blob/HEAD/docs/funcs_migration.md"]
pub trait SystemTableView {}

/// Marker struct associated with the boot view of the UEFI System Table.
#[deprecated = "Use the uefi::boot module instead. See https://github.com/rust-osdev/uefi-rs/blob/HEAD/docs/funcs_migration.md"]
#[derive(Debug)]
pub struct Boot;
impl SystemTableView for Boot {}

/// Marker struct associated with the run-time view of the UEFI System Table.
#[deprecated = "Use the uefi::runtime module instead. See https://github.com/rust-osdev/uefi-rs/blob/HEAD/docs/funcs_migration.md"]
#[derive(Debug)]
pub struct Runtime;
impl SystemTableView for Runtime {}

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

    /// Access runtime services
    #[must_use]
    pub const fn runtime_services(&self) -> &RuntimeServices {
        unsafe { &*(*self.table).runtime_services.cast_const().cast() }
    }

    /// Access boot services
    #[must_use]
    pub const fn boot_services(&self) -> &BootServices {
        unsafe { &*(*self.table).boot_services.cast_const().cast() }
    }

    /// Get the current memory map and exit boot services.
    unsafe fn get_memory_map_and_exit_boot_services(
        &self,
        buf: &mut [u8],
    ) -> Result<MemoryMapMeta> {
        let boot_services = self.boot_services();

        // Get the memory map.
        let memory_map = boot_services.get_memory_map(buf)?;

        // Try to exit boot services using the memory map key. Note that after
        // the first call to `exit_boot_services`, there are restrictions on
        // what boot services functions can be called. In UEFI 2.8 and earlier,
        // only `get_memory_map` and `exit_boot_services` are allowed. Starting
        // in UEFI 2.9 other memory allocation functions may also be called.
        boot_services
            .exit_boot_services(boot_services.image_handle(), memory_map.map_key)
            .map(move |()| memory_map)
    }

    /// Exit the UEFI boot services.
    ///
    /// After this function completes, UEFI hands over control of the hardware
    /// to the executing OS loader, which implies that the UEFI boot services
    /// are shut down and cannot be used anymore. Only UEFI configuration tables
    /// and run-time services can be used, and the latter requires special care
    /// from the OS loader. We model this situation by consuming the
    /// `SystemTable<Boot>` view of the System Table and returning a more
    /// restricted `SystemTable<Runtime>` view as an output.
    ///
    /// The memory map at the time of exiting boot services is also
    /// returned. The map is backed by a allocation with given `memory_type`.
    /// Since the boot services function to free that memory is no
    /// longer available after calling `exit_boot_services`, the allocation is
    /// live until the program ends. The lifetime of the memory map is therefore
    /// `'static`.
    ///
    /// Note that once the boot services are exited, associated loggers and
    /// allocators can't use the boot services anymore. For the corresponding
    /// abstractions provided by this crate (see the [`helpers`] module),
    /// invoking this function will automatically disable them. If the
    /// `global_allocator` feature is enabled, attempting to use the allocator
    /// after exiting boot services will panic.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring that no references to
    /// boot-services data remain. A non-exhaustive list of resources to check:
    ///
    /// * All protocols will be invalid after exiting boot services. This
    ///   includes the [`Output`] protocols attached to stdout/stderr. The
    ///   caller must ensure that no protocol references remain.
    /// * The pool allocator is not usable after exiting boot services. Types
    ///   such as [`PoolString`] which call [`BootServices::free_pool`] on drop
    ///   must be cleaned up before calling `exit_boot_services`, or leaked to
    ///   avoid drop ever being called.
    /// * All data in the memory map marked as
    ///   [`MemoryType::BOOT_SERVICES_CODE`] and
    ///   [`MemoryType::BOOT_SERVICES_DATA`] will become free memory, the caller
    ///   must ensure that no references to such memory exist.
    ///
    /// # Errors
    ///
    /// This function will fail if it is unable to allocate memory for
    /// the memory map, if it fails to retrieve the memory map, or if
    /// exiting boot services fails (with up to one retry).
    ///
    /// All errors are treated as unrecoverable because the system is
    /// now in an undefined state. Rather than returning control to the
    /// caller, the system will be reset.
    ///
    /// [`helpers`]: crate::helpers
    /// [`Output`]: crate::proto::console::text::Output
    /// [`PoolString`]: crate::proto::device_path::text::PoolString
    #[must_use]
    pub unsafe fn exit_boot_services(
        self,
        memory_type: MemoryType,
    ) -> (SystemTable<Runtime>, MemoryMapOwned) {
        crate::helpers::exit();

        // Reboot the device.
        let reset = |status| -> ! {
            {
                log::warn!("Resetting the machine");
                self.runtime_services().reset(ResetType::COLD, status, None)
            }
        };

        let mut buf = MemoryMapBackingMemory::new(memory_type).expect("Failed to allocate memory");

        // Calling `exit_boot_services` can fail if the memory map key is not
        // current. Retry a second time if that occurs. This matches the
        // behavior of the Linux kernel:
        // https://github.com/torvalds/linux/blob/e544a0743/drivers/firmware/efi/libstub/efi-stub-helper.c#L375
        let mut status = Status::ABORTED;
        for _ in 0..2 {
            match unsafe { self.get_memory_map_and_exit_boot_services(buf.as_mut_slice()) } {
                Ok(memory_map) => {
                    let st = SystemTable {
                        table: self.table,
                        _marker: PhantomData,
                    };
                    return (st, MemoryMapOwned::from_initialized_mem(buf, memory_map));
                }
                Err(err) => {
                    log::error!("Error retrieving the memory map for exiting the boot services");
                    status = err.status()
                }
            }
        }

        // Failed to exit boot services.
        reset(status)
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
