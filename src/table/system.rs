use super::boot::BootServices;
use super::runtime::RuntimeServices;
use super::{cfg, Header, Revision};
use core::slice;
use crate::proto::console::text;
use crate::{CStr16, Char16, Handle};


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

    // TODO: Provide a way to exit boot services and get a RuntimeSystemTable,
    //       consuming the BootSystemTable in the process. The interface must
    //       allow get_memory_map calls and calls to run-time services, but no
    //       calls to boot-time services since these can be shut down after the
    //       first attempt.

    /// Clone this UEFI system table handle
    ///
    /// This is unsafe because you must guarantee that the clone will not be
    /// used after boot services are exited. However, the singleton-based
    /// designs that Rust uses for memory allocation, logging, and panic
    /// handling require taking this risk.
    pub unsafe fn clone(&self) -> Self {
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
#[repr(transparent)]
pub struct RuntimeSystemTable(&'static SystemTable);

// TODO: Provide a RuntimeSystemTable, paying attention to what's now unsafe


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
