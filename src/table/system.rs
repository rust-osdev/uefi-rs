use super::boot::{BootServices, MemoryMapIter, MemoryMapKey};
use super::runtime::RuntimeServices;
use super::{cfg, Header, Revision};
use core::marker::PhantomData;
use core::slice;
use crate::proto::console::text;
use crate::{CStr16, Char16, Handle, Result, ResultExt, Status};

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
}

// These parts of the UEFI System Table interface may only be used until boot
// services are exited and hardware control is handed over to the OS loader
#[allow(clippy::mut_from_ref)]
impl SystemTable<Boot> {
    /// Returns the standard input protocol.
    pub fn stdin(&self) -> &mut text::Input {
        unsafe { &mut *self.table.stdin }
    }

    /// Returns the standard output protocol.
    pub fn stdout(&self) -> &mut text::Output {
        let stdout_ptr = self.table.stdout as *const _ as *mut _;
        unsafe { &mut *stdout_ptr }
    }

    /// Returns the standard error protocol.
    pub fn stderr(&self) -> &mut text::Output {
        let stderr_ptr = self.table.stderr as *const _ as *mut _;
        unsafe { &mut *stderr_ptr }
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
    /// After this function completes, the UEFI boot services are shut down and
    /// cannot be used anymore. Only run-time services can be used. We model
    /// this by consuming the SystemTable<Boot> view and returning a more
    /// restricted SystemTable<Runtime> view as an output.
    ///
    /// The handle passed must be the one of the currently executing image,
    /// which is received by the entry point of the UEFI application. In
    /// addition, the application **must** retrieve the current memory map, and
    /// pass the associated MemoryMapKey so that the firmware can check that it
    /// is up to date.
    ///
    /// If the application's memory map is not up to date, then the firmware
    /// ends up in an awkward situation where it may have already shut down some
    /// of the boot services, but it wants to give the application a chance to
    /// fetch a newer memory map. This is modeled in the API by the
    /// `retry_handler`, a user-provided functor which is given as input...
    ///
    /// - The new size of the memory map
    /// - Access to the `memory_map` boot service (as described in the
    ///   `BootServices` table)
    ///
    /// The functor _may_ attempt to use this information to fetch the memory
    /// map again and return the associated key. It will be repeatedly called
    /// by `exit_boot_services` implementation as long as it does so but the
    /// output MemoryMapKey is stale. If the functor chooses to return `None`
    /// instead, `exit_boot_services` will terminate with an error.
    ///
    /// If `exit_boot_services` succeeds, it will return a runtime view of the
    /// system table which more accurately reflects the state of the UEFI
    /// firmware following exit from boot services.
    pub fn exit_boot_services(
        self,
        image: Handle,
        mmap_key: MemoryMapKey,
        mut retry_handler: impl FnMut(
            // Size of the new memory map
            usize,
            // Access to the `memory_map` boot service
            &mut FnMut(&mut [u8]) -> Result<(MemoryMapKey, MemoryMapIter)>,
        ) -> Option<MemoryMapKey>,
    ) -> Result<SystemTable<Runtime>> {
        unsafe {
            let boot_services = self.boot_services();

            // Try to exit the UEFI boot services
            let mut result = boot_services.exit_boot_services(image, mmap_key);

            // If the MapKey is incorrect, give the user a chance to update it
            while let Err(Status::INVALID_PARAMETER) = result {
                // Call the user's retry handler
                let mmap_key_opt = retry_handler(boot_services.memory_map_size(), &mut |buf| {
                    boot_services.memory_map(buf)
                });

                // Check if the retry handler provided a new mmap key or gave up
                if let Some(mmap_key) = mmap_key_opt {
                    result = boot_services.exit_boot_services(image, mmap_key);
                } else {
                    break;
                }
            }

            // Declare success or failure
            result.map_inner(|_| SystemTable {
                table: self.table,
                _marker: PhantomData,
            })
        }
    }

    /// Clone this boot-time UEFI system table interface
    ///
    /// This is unsafe because you must guarantee that the clone will not be
    /// used after boot services are exited. However, the singleton-based
    /// designs that Rust uses for memory allocation, logging, and panic
    /// handling require taking this risk.
    pub unsafe fn unsafe_clone(&self) -> Self {
        SystemTable {
            table: self.table,
            _marker: PhantomData,
        }
    }
}

// These parts of the UEFI System Table interface may only be used after exit
// from UEFI boot services
impl SystemTable<Runtime> {
    /// Access runtime services
    ///
    /// This is unsafe because UEFI runtime services require an elaborate
    /// CPU configuration which may not be preserved by OS loaders. See the
    /// "Calling Conventions" chapter of the UEFI specification for details.
    pub unsafe fn runtime_services(&self) -> &RuntimeServices {
        self.table.runtime
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
    /// Number of entires in the configuration table.
    nr_cfg: usize,
    /// Pointer to beginning of the array.
    cfg_table: *const cfg::ConfigTableEntry,
}

impl<View: SystemTableView> super::Table for SystemTable<View> {
    const SIGNATURE: u64 = 0x5453_5953_2049_4249;
}
