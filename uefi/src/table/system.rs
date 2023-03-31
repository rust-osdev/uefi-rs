use core::ffi::c_void;
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
use core::ptr::NonNull;
use core::{ptr, slice};

use crate::proto::console::text;
use crate::{CStr16, Char16, Handle, Result, Status};

use super::boot::{BootServices, MemoryDescriptor, MemoryMap, MemoryType};
use super::runtime::{ResetType, RuntimeServices};
use super::{cfg, Header, Revision};

/// Marker trait used to provide different views of the UEFI System Table.
pub trait SystemTableView {}

/// Marker struct associated with the boot view of the UEFI System Table.
#[derive(Debug)]
pub struct Boot;
impl SystemTableView for Boot {}

/// Marker struct associated with the run-time view of the UEFI System Table.
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
/// will be provided to replace it.
#[repr(transparent)]
pub struct SystemTable<View: SystemTableView> {
    table: *const SystemTableImpl,
    _marker: PhantomData<View>,
}

// These parts of the UEFI System Table interface will always be available
impl<View: SystemTableView> SystemTable<View> {
    /// Return the firmware vendor string
    #[must_use]
    pub fn firmware_vendor(&self) -> &CStr16 {
        unsafe { CStr16::from_ptr((*self.table).fw_vendor) }
    }

    /// Return the firmware revision
    #[must_use]
    pub const fn firmware_revision(&self) -> u32 {
        unsafe { (*self.table).fw_revision }
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
        unsafe { slice::from_raw_parts((*self.table).cfg_table, (*self.table).nr_cfg) }
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
}

// These parts of the UEFI System Table interface may only be used until boot
// services are exited and hardware control is handed over to the OS loader
impl SystemTable<Boot> {
    /// Returns the standard input protocol.
    pub fn stdin(&mut self) -> &mut text::Input {
        unsafe { &mut *(*self.table).stdin }
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
        unsafe { &*(*self.table).runtime }
    }

    /// Access boot services
    #[must_use]
    pub const fn boot_services(&self) -> &BootServices {
        unsafe { &*(*self.table).boot }
    }

    /// Get the size in bytes of the buffer to allocate for storing the memory
    /// map in `exit_boot_services`.
    ///
    /// This map contains some extra room to avoid needing to allocate more than
    /// once.
    ///
    /// Returns `None` on overflow.
    fn memory_map_size_for_exit_boot_services(&self) -> Option<usize> {
        // Allocate space for extra entries beyond the current size of the
        // memory map. The value of 8 matches the value in the Linux kernel:
        // https://github.com/torvalds/linux/blob/e544a07438/drivers/firmware/efi/libstub/efistub.h#L173
        let extra_entries = 8;

        let memory_map_size = self.boot_services().memory_map_size();
        let extra_size = memory_map_size.entry_size.checked_mul(extra_entries)?;
        memory_map_size.map_size.checked_add(extra_size)
    }

    /// Get the current memory map and exit boot services.
    unsafe fn get_memory_map_and_exit_boot_services(
        &self,
        buf: &'static mut [u8],
    ) -> Result<MemoryMap<'static>> {
        let boot_services = self.boot_services();

        // Get the memory map.
        let memory_map = boot_services.memory_map(buf)?;

        // Try to exit boot services using the memory map key. Note that after
        // the first call to `exit_boot_services`, there are restrictions on
        // what boot services functions can be called. In UEFI 2.8 and earlier,
        // only `get_memory_map` and `exit_boot_services` are allowed. Starting
        // in UEFI 2.9 other memory allocation functions may also be called.
        boot_services
            .exit_boot_services(boot_services.image_handle(), memory_map.key())
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
    /// returned. The map is backed by a [`MemoryType::LOADER_DATA`]
    /// allocation. Since the boot services function to free that memory is no
    /// longer available after calling `exit_boot_services`, the allocation is
    /// live until the program ends. The lifetime of the memory map is therefore
    /// `'static`.
    ///
    /// Once boot services are exited, the logger and allocator provided by
    /// this crate can no longer be used. The logger should be disabled using
    /// the [`Logger::disable`] method, and the allocator should be disabled by
    /// calling [`allocator::exit_boot_services`]. Note that if the logger and
    /// allocator were initialized with [`uefi_services::init`], they will be
    /// disabled automatically when `exit_boot_services` is called.
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
    /// [`allocator::exit_boot_services`]: crate::allocator::exit_boot_services
    /// [`Logger::disable`]: crate::logger::Logger::disable
    /// [`uefi_services::init`]: https://docs.rs/uefi-services/latest/uefi_services/fn.init.html
    #[must_use]
    pub fn exit_boot_services(self) -> (SystemTable<Runtime>, MemoryMap<'static>) {
        let boot_services = self.boot_services();

        // Reboot the device.
        let reset = |status| -> ! { self.runtime_services().reset(ResetType::Cold, status, None) };

        // Get the size of the buffer to allocate. If that calculation
        // overflows treat it as an unrecoverable error.
        let buf_size = match self.memory_map_size_for_exit_boot_services() {
            Some(buf_size) => buf_size,
            None => reset(Status::ABORTED),
        };

        // Allocate a byte slice to hold the memory map. If the
        // allocation fails treat it as an unrecoverable error.
        let buf: *mut u8 = match boot_services.allocate_pool(MemoryType::LOADER_DATA, buf_size) {
            Ok(buf) => buf,
            Err(err) => reset(err.status()),
        };

        // Calling `exit_boot_services` can fail if the memory map key is not
        // current. Retry a second time if that occurs. This matches the
        // behavior of the Linux kernel:
        // https://github.com/torvalds/linux/blob/e544a0743/drivers/firmware/efi/libstub/efi-stub-helper.c#L375
        let mut status = Status::ABORTED;
        for _ in 0..2 {
            let buf: &mut [u8] = unsafe { slice::from_raw_parts_mut(buf, buf_size) };
            match unsafe { self.get_memory_map_and_exit_boot_services(buf) } {
                Ok(memory_map) => {
                    let st = SystemTable {
                        table: self.table,
                        _marker: PhantomData,
                    };
                    return (st, memory_map);
                }
                Err(err) => status = err.status(),
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
        SystemTable {
            table: self.table,
            _marker: PhantomData,
        }
    }
}

impl Debug for SystemTable<Boot> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        unsafe { &*self.table }.fmt(f)
    }
}

// These parts of the SystemTable struct are only visible after exit from UEFI
// boot services. They provide unsafe access to the UEFI runtime services, which
// which were already available before but in safe form.
impl SystemTable<Runtime> {
    /// Access runtime services
    ///
    /// # Safety
    ///
    /// This is unsafe because UEFI runtime services require an elaborate
    /// CPU configuration which may not be preserved by OS loaders. See the
    /// "Calling Conventions" chapter of the UEFI specification for details.
    #[must_use]
    pub const unsafe fn runtime_services(&self) -> &RuntimeServices {
        &*(*self.table).runtime
    }

    /// Changes the runtime addressing mode of EFI firmware from physical to virtual.
    /// It is up to the caller to translate the old SystemTable address to a new virtual
    /// address and provide it for this function.
    /// See [`get_current_system_table_addr`]
    ///
    /// # Safety
    ///
    /// Setting new virtual memory map is unsafe and may cause undefined behaviors.
    ///
    /// [`get_current_system_table_addr`]: SystemTable::get_current_system_table_addr
    pub unsafe fn set_virtual_address_map(
        self,
        map: &mut [MemoryDescriptor],
        new_system_table_virtual_addr: u64,
    ) -> Result<Self> {
        // Unsafe Code Guidelines guarantees that there is no padding in an array or a slice
        // between its elements if the element type is `repr(C)`, which is our case.
        //
        // See https://rust-lang.github.io/unsafe-code-guidelines/layout/arrays-and-slices.html
        let map_size = core::mem::size_of_val(map);
        let entry_size = core::mem::size_of::<MemoryDescriptor>();
        let entry_version = crate::table::boot::MEMORY_DESCRIPTOR_VERSION;
        let map_ptr = map.as_mut_ptr();
        ((*(*self.table).runtime).set_virtual_address_map)(
            map_size,
            entry_size,
            entry_version,
            map_ptr,
        )
        .into_with_val(|| {
            let new_table_ref =
                &mut *(new_system_table_virtual_addr as usize as *mut SystemTableImpl);
            Self {
                table: new_table_ref,
                _marker: PhantomData,
            }
        })
    }

    /// Return the address of the SystemTable that resides in a UEFI runtime services
    /// memory region.
    #[must_use]
    pub fn get_current_system_table_addr(&self) -> u64 {
        self.table as u64
    }
}

/// The actual UEFI system table
#[repr(C)]
struct SystemTableImpl {
    header: Header,
    /// Null-terminated string representing the firmware's vendor.
    fw_vendor: *const Char16,
    fw_revision: u32,
    stdin_handle: Handle,
    stdin: *mut text::Input,
    stdout_handle: Handle,
    stdout: *mut text::Output,
    stderr_handle: Handle,
    stderr: *mut text::Output,
    /// Runtime services table.
    runtime: *const RuntimeServices,
    /// Boot services table.
    boot: *const BootServices,
    /// Number of entries in the configuration table.
    nr_cfg: usize,
    /// Pointer to beginning of the array.
    cfg_table: *const cfg::ConfigTableEntry,
}

impl<View: SystemTableView> super::Table for SystemTable<View> {
    const SIGNATURE: u64 = 0x5453_5953_2049_4249;
}

impl Debug for SystemTableImpl {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UefiSystemTable")
            .field("header", &self.header)
            .field("fw_vendor", &(unsafe { CStr16::from_ptr(self.fw_vendor) }))
            .field("fw_revision", &self.fw_revision)
            .field("stdin_handle", &self.stdin_handle)
            .field("stdin", &self.stdin)
            .field("stdout_handle", &self.stdout_handle)
            .field("stdout", &self.stdout)
            .field("stderr_handle", &self.stderr_handle)
            .field("stderr", &self.stderr)
            .field("runtime", &self.runtime)
            // a little bit of extra work needed to call debug-fmt on the BootServices
            // instead of printing the raw pointer
            .field("boot", &(unsafe { ptr::read(self.boot) }))
            .field("nf_cfg", &self.nr_cfg)
            .field("cfg_table", &self.cfg_table)
            .finish()
    }
}
