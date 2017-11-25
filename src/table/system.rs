use Handle;
use super::{Header, Revision, cfg};
use proto::console::text;
use core::slice;

/// The system table entry points for accessing the core UEFI system functionality.
#[repr(C)]
pub struct SystemTable {
    header: Header,
    /// Null-terminated string representing the firmware's vendor.
    pub fw_vendor: *const u16,
    /// Revision of the UEFI specification the firmware conforms to.
    pub fw_revision: Revision,
    stdin_handle: Handle,
    stdin: *mut text::Input,
    stdout_handle: Handle,
    stdout: *mut text::Output,
    stderr_handle: Handle,
    stderr: *mut text::Output,
    /// Runtime services table.
    pub runtime: &'static super::runtime::RuntimeServices,
    /// Boot services table.
    pub boot: &'static super::boot::BootServices,
    /// Number of entires in the configuration table.
    nr_cfg: usize,
    /// Pointer to beginning of the array.
    cfg_table: *mut cfg::ConfigTableEntry,
}

impl SystemTable {
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
        unsafe { &mut *self.stdout }
    }

    /// Returns the standard error protocol.
    pub fn stderr(&self) -> &mut text::Output {
        unsafe { &mut *self.stderr }
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
