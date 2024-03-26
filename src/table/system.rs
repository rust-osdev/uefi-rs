use core::ffi::c_void;
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
use core::ptr::NonNull;
use core::{ptr, slice};

use crate::proto::console::text;
use crate::{CStr16, Char16, Handle, Result, ResultExt, Status};

use super::boot::{BootServices, MemoryDescriptor};
use super::runtime::RuntimeServices;
use super::{cfg, Header, Revision};

/// Marker trait used to provide different views of the UEFI System Table
pub trait SystemTableView {}

/// Marker struct associated with the boot view of the UEFI System Table
pub struct Boot;
impl SystemTableView for Boot {}

/// Marker struct associated with the run-time view of the UEFI System Table
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
#[derive(Debug)]
pub struct SystemTable<View: SystemTableView> {
    table: &'static SystemTableImpl,
    _marker: PhantomData<View>,
}

// These parts of the UEFI System Table interface will always be available
impl<View: SystemTableView> SystemTable<View> {
    /// Return the firmware vendor string
    pub fn firmware_vendor(&self) -> &CStr16 {
        unsafe { CStr16::from_ptr(self.table.fw_vendor) }
    }

    /// Return the firmware revision
    pub fn firmware_revision(&self) -> Revision {
        self.table.fw_revision
    }

    /// Returns the revision of this table, which is defined to be
    /// the revision of the UEFI specification implemented by the firmware.
    pub fn uefi_revision(&self) -> Revision {
        self.table.header.revision
    }

    /// Returns the config table entries, a linear array of structures
    /// pointing to other system-specific tables.
    pub fn config_table(&self) -> &[cfg::ConfigTableEntry] {
        unsafe { slice::from_raw_parts(self.table.cfg_table, self.table.nr_cfg) }
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
        unsafe { &mut *self.table.stdin }
    }

    /// Returns the standard output protocol.
    pub fn stdout(&mut self) -> &mut text::Output {
        unsafe { &mut *self.table.stdout.cast() }
    }

    /// Returns the standard error protocol.
    pub fn stderr(&mut self) -> &mut text::Output {
        unsafe { &mut *self.table.stderr.cast() }
    }

    /// Access runtime services
    pub fn runtime_services(&self) -> &RuntimeServices {
        self.table.runtime
    }

    /// Access boot services
    pub fn boot_services(&self) -> &BootServices {
        unsafe { &*self.table.boot }
    }

    /// Exit the UEFI boot services
    ///
    /// After this function completes, UEFI hands over control of the hardware
    /// to the executing OS loader, which implies that the UEFI boot services
    /// are shut down and cannot be used anymore. Only UEFI configuration tables
    /// and run-time services can be used, and the latter requires special care
    /// from the OS loader. We model this situation by consuming the
    /// `SystemTable<Boot>` view of the System Table and returning a more
    /// restricted `SystemTable<Runtime>` view as an output.
    ///
    /// Once boot services are exited, the logger and allocator provided by
    /// this crate can no longer be used. The logger should be disabled using
    /// the [`Logger::disable`] method, and the allocator should be disabled by
    /// calling [`alloc::exit_boot_services`]. Note that if the logger and
    /// allocator were initialized with [`uefi_services::init`], they will be
    /// disabled automatically when `exit_boot_services` is called.
    ///
    /// The handle passed must be the one of the currently executing image,
    /// which is received by the entry point of the UEFI application. In
    /// addition, the application must provide storage for a memory map, which
    /// will be retrieved automatically (as having an up-to-date memory map is a
    /// prerequisite for exiting UEFI boot services).
    ///
    /// The storage must be aligned like a `MemoryDescriptor`.
    ///
    /// The size of the memory map can be estimated by calling
    /// `BootServices::memory_map_size()`. But the memory map can grow under the
    /// hood between the moment where this size estimate is returned and the
    /// moment where boot services are exited, and calling the UEFI memory
    /// allocator will not be possible after the first attempt to exit the boot
    /// services. Therefore, UEFI applications are advised to allocate storage
    /// for the memory map right before exiting boot services, and to allocate a
    /// bit more storage than requested by memory_map_size.
    ///
    /// If `exit_boot_services` succeeds, it will return a runtime view of the
    /// system table which more accurately reflects the state of the UEFI
    /// firmware following exit from boot services, along with a high-level
    /// iterator to the UEFI memory map.
    ///
    /// [`alloc::exit_boot_services`]: crate::alloc::exit_boot_services
    /// [`Logger::disable`]: crate::logger::Logger::disable
    /// [`uefi_services::init`]: https://docs.rs/uefi-services/latest/uefi_services/fn.init.html
    pub fn exit_boot_services(
        self,
        image: Handle,
        mmap_buf: &mut [u8],
    ) -> Result<(
        SystemTable<Runtime>,
        impl ExactSizeIterator<Item = &MemoryDescriptor> + Clone,
    )> {
        unsafe {
            let boot_services = self.boot_services();

            loop {
                // Fetch a memory map, propagate errors and split the completion
                // FIXME: This sad pointer hack works around a current
                //        limitation of the NLL analysis (see Rust bug 51526).
                let mmap_buf = &mut *(mmap_buf as *mut [u8]);
                let mmap_comp = boot_services.memory_map(mmap_buf)?;
                let (mmap_key, mmap_iter) = mmap_comp;

                // Try to exit boot services using this memory map key
                let result = boot_services.exit_boot_services(image, mmap_key);

                // Did we fail because the memory map was updated concurrently?
                if result.status() == Status::INVALID_PARAMETER {
                    // If so, fetch another memory map and try again
                    continue;
                } else {
                    // If not, report the outcome of the operation
                    return result.map(|_| {
                        let st = SystemTable {
                            table: self.table,
                            _marker: PhantomData,
                        };
                        (st, mmap_iter)
                    });
                }
            }
        }
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
    pub unsafe fn unsafe_clone(&self) -> Self {
        SystemTable {
            table: self.table,
            _marker: PhantomData,
        }
    }
}

impl Debug for SystemTable<Boot> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.table.fmt(f)
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
    pub unsafe fn runtime_services(&self) -> &RuntimeServices {
        self.table.runtime
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
        (self.table.runtime.set_virtual_address_map)(map_size, entry_size, entry_version, map_ptr)
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
    pub fn get_current_system_table_addr(&self) -> u64 {
        self.table as *const _ as usize as u64
    }
}

/// The actual UEFI system table
#[repr(C)]
struct SystemTableImpl {
    header: Header,
    /// Null-terminated string representing the firmware's vendor.
    fw_vendor: *const Char16,
    /// Revision of the UEFI specification the firmware conforms to.
    fw_revision: Revision,
    stdin_handle: Handle,
    stdin: *mut text::Input,
    stdout_handle: Handle,
    stdout: *mut text::Output<'static>,
    stderr_handle: Handle,
    stderr: *mut text::Output<'static>,
    /// Runtime services table.
    runtime: &'static RuntimeServices,
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
