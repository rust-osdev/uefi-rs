use super::boot::BootServices;
use super::runtime::RuntimeServices;
use super::{cfg, Header, Revision};
use core::slice;
use crate::proto::console::text;
use crate::{CStr16, Char16, Handle};

/// The system table entry points for accessing the core UEFI system functionality.
#[repr(C)]
pub struct SystemTable {
    header: Header,
    /// Null-terminated string representing the firmware's vendor.
    fw_vendor: *const Char16,
    /// Revision of the UEFI specification the firmware conforms to.
    pub fw_revision: Revision,
    stdin_handle: Handle,
    stdin: *mut text::Input,
    stdout_handle: Handle,
    stdout: *mut text::Output<'static>,
    stderr_handle: Handle,
    stderr: *mut text::Output<'static>,
    /// Runtime services table.
    runtime: &'static RuntimeServices,
    /// Boot services table.
    boot: &'static BootServices,
    /// Number of entires in the configuration table.
    nr_cfg: usize,
    /// Pointer to beginning of the array.
    cfg_table: *const cfg::ConfigTableEntry,
}

// This is unsafe, but it's the best solution we have from now.
#[allow(clippy::mut_from_ref)]
impl SystemTable {
    /// Return the firmware vendor string
    pub fn firmware_vendor(&self) -> &CStr16 {
        unsafe { CStr16::from_ptr(self.fw_vendor) }
    }

    /// Returns the revision of this table, which is defined to be
    /// the revision of the UEFI specification implemented by the firmware.
    pub fn uefi_revision(&self) -> Revision {
        self.header.revision
    }

    /// Returns the standard input protocol.
    pub fn stdin(&self) -> &mut text::Input {
        unsafe { &mut *self.stdin }
    }

    /// Returns the standard output protocol.
    pub fn stdout(&self) -> &mut text::Output {
        let stdout_ptr = self.stdout as *const _ as *mut _;
        unsafe { &mut *stdout_ptr }
    }

    /// Returns the standard error protocol.
    pub fn stderr(&self) -> &mut text::Output {
        let stderr_ptr = self.stderr as *const _ as *mut _;
        unsafe { &mut *stderr_ptr }
    }

    /// Access runtime services
    pub fn runtime_services(&self) -> &RuntimeServices {
        self.runtime
    }

    /// Access boot services
    pub fn boot_services(&self) -> &BootServices {
        self.boot
    }

    /// Returns the config table entries, a linear array of structures
    /// pointing to other system-specific tables.
    pub fn config_table(&self) -> &[cfg::ConfigTableEntry] {
        unsafe { slice::from_raw_parts(self.cfg_table, self.nr_cfg) }
    }
}

impl super::Table for SystemTable {
    const SIGNATURE: u64 = 0x5453_5953_2049_4249;
}
