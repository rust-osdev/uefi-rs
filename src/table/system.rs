use super::boot::{BootServices, MemoryMapIter, MemoryMapKey};
use super::runtime::RuntimeServices;
use super::{cfg, Header, Revision};
use core::slice;
use crate::proto::console::text;
use crate::{CStr16, Char16, Handle, Result, ResultExt, Status};

/// Boot-time version of the UEFI System Table
///
/// This is the view of the UEFI System Table that an UEFI application is
/// provided with initially. It enables calling all UEFI services, including
/// so-called boot services, and exchanging information with the standard input,
/// output and error streams.
///
/// Once an UEFI OS loader is ready to take over the system, it can call
/// `exit_boot_services` to terminate the UEFI boot services. This will consume
/// the BootSystemTable (enabling the Rust borrow checker to tell you about any
/// boot service handle that you forgot about) and give you a RuntimeSystemTable
/// which is more appropriate for post-boot usage.
#[repr(transparent)]
pub struct BootSystemTable(&'static SystemTable);

// This is unsafe, but it's the best solution we have from now.
#[allow(clippy::mut_from_ref)]
impl BootSystemTable {
    /// Return the firmware vendor string
    pub fn firmware_vendor(&self) -> &CStr16 {
        unsafe { CStr16::from_ptr(self.0.fw_vendor) }
    }

    /// Return the firmware revision
    pub fn firmware_revision(&self) -> Revision {
        self.0.fw_revision
    }

    /// Returns the revision of this table, which is defined to be
    /// the revision of the UEFI specification implemented by the firmware.
    pub fn uefi_revision(&self) -> Revision {
        self.0.header.revision
    }

    /// Returns the standard input protocol.
    pub fn stdin(&self) -> &mut text::Input {
        unsafe { &mut *self.0.stdin }
    }

    /// Returns the standard output protocol.
    pub fn stdout(&self) -> &mut text::Output {
        let stdout_ptr = self.0.stdout as *const _ as *mut _;
        unsafe { &mut *stdout_ptr }
    }

    /// Returns the standard error protocol.
    pub fn stderr(&self) -> &mut text::Output {
        let stderr_ptr = self.0.stderr as *const _ as *mut _;
        unsafe { &mut *stderr_ptr }
    }

    /// Access runtime services
    pub fn runtime_services(&self) -> &RuntimeServices {
        self.0.runtime
    }

    /// Access boot services
    pub fn boot_services(&self) -> &BootServices {
        unsafe { &*self.0.boot }
    }

    /// Returns the config table entries, a linear array of structures
    /// pointing to other system-specific tables.
    pub fn config_table(&self) -> &[cfg::ConfigTableEntry] {
        unsafe { slice::from_raw_parts(self.0.cfg_table, self.0.nr_cfg) }
    }

    /// Exit the UEFI boot services
    ///
    /// After this function completes, the UEFI boot services are shut down and
    /// cannot be used anymore. Only run-time services can be used. We model
    /// this by consuming the BootSystemTable and returning a more restricted
    /// RuntimeSystemTable as an output.
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
    /// If `exit_boot_services` succeeds, it will return a `RuntimeSystemTable`,
    /// which is a different view of the UEFI System Table that is more accurate
    /// than the `BootSystemTable` view after boot services have been exited.
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
    ) -> Result<RuntimeSystemTable> {
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
            result.map_inner(|_| RuntimeSystemTable(self.0))
        }
    }

    /// Clone this boot-time UEFI system table interface
    ///
    /// This is unsafe because you must guarantee that the clone will not be
    /// used after boot services are exited. However, the singleton-based
    /// designs that Rust uses for memory allocation, logging, and panic
    /// handling require taking this risk.
    pub unsafe fn unsafe_clone(&self) -> Self {
        BootSystemTable(self.0)
    }
}

/// Run-time version of the UEFI System Table
///
/// This is the view of the UEFI System Table that an UEFI application has after
/// exiting the boot services.
///
/// It does not expose functionality which is unavailable after exiting boot
/// services, and actions which are very likely to become unsafe after an
/// operating system has started initializing are marked as such.
pub struct RuntimeSystemTable(&'static SystemTable);

// This is unsafe, but it's the best solution we have from now.
#[allow(clippy::mut_from_ref)]
impl RuntimeSystemTable {
    /// Return the firmware vendor string
    pub fn firmware_vendor(&self) -> &CStr16 {
        unsafe { CStr16::from_ptr(self.0.fw_vendor) }
    }

    /// Return the firmware revision
    pub fn firmware_revision(&self) -> Revision {
        self.0.fw_revision
    }

    /// Returns the revision of this table, which is defined to be
    /// the revision of the UEFI specification implemented by the firmware.
    pub fn uefi_revision(&self) -> Revision {
        self.0.header.revision
    }

    /// Access runtime services
    ///
    /// This is unsafe because UEFI runtime services require an elaborate
    /// CPU configuration which may not be preserved by OS loaders. See the
    /// "Calling Conventions" chapter of the UEFI specification for details.
    pub unsafe fn runtime_services(&self) -> &RuntimeServices {
        self.0.runtime
    }

    /// Returns the config table entries, a linear array of structures
    /// pointing to other system-specific tables.
    pub fn config_table(&self) -> &[cfg::ConfigTableEntry] {
        unsafe { slice::from_raw_parts(self.0.cfg_table, self.0.nr_cfg) }
    }
}

/// The system table entry points for accessing the core UEFI system functionality.
#[repr(C)]
struct SystemTable {
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

impl super::Table for SystemTable {
    const SIGNATURE: u64 = 0x5453_5953_2049_4249;
}
