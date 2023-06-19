use crate::protocol::console::{SimpleTextInputProtocol, SimpleTextOutputProtocol};
use crate::table::boot::BootServices;
use crate::table::configuration::ConfigurationTable;
use crate::table::runtime::RuntimeServices;
use crate::table::Header;
use crate::{Char16, Handle};
use core::{mem, ptr};

#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct SystemTable {
    pub header: Header,

    pub firmware_vendor: *const Char16,
    pub firmware_revision: u32,

    pub stdin_handle: Handle,
    pub stdin: *mut SimpleTextInputProtocol,

    pub stdout_handle: Handle,
    pub stdout: *mut SimpleTextOutputProtocol,

    pub stderr_handle: Handle,
    pub stderr: *mut SimpleTextOutputProtocol,

    pub runtime_services: *mut RuntimeServices,
    pub boot_services: *mut BootServices,

    pub number_of_configuration_table_entries: usize,
    pub configuration_table: *mut ConfigurationTable,
}

impl SystemTable {
    pub const SIGNATURE: u64 = 0x5453_5953_2049_4249;
}

impl Default for SystemTable {
    /// Create a `SystemTable` with most fields set to zero.
    ///
    /// The only fields not set to zero are:
    /// * [`Header::signature`] is set to [`SystemTable::SIGNATURE`].
    /// * [`Header::size`] is set to the size in bytes of `SystemTable`.
    fn default() -> Self {
        Self {
            header: Header {
                signature: Self::SIGNATURE,
                size: u32::try_from(mem::size_of::<Self>()).unwrap(),
                ..Header::default()
            },

            firmware_vendor: ptr::null_mut(),
            firmware_revision: 0,

            stdin_handle: ptr::null_mut(),
            stdin: ptr::null_mut(),

            stdout_handle: ptr::null_mut(),
            stdout: ptr::null_mut(),

            stderr_handle: ptr::null_mut(),
            stderr: ptr::null_mut(),

            runtime_services: ptr::null_mut(),
            boot_services: ptr::null_mut(),

            number_of_configuration_table_entries: 0,
            configuration_table: ptr::null_mut(),
        }
    }
}
