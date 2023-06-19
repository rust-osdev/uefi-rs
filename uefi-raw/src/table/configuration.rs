use crate::Guid;
use core::ffi::c_void;

/// UEFI configuration table.
///
/// Each table is uniquely identified by a GUID. The type of data pointed to by
/// `vendor_table`, as well as whether that address is physical or virtual,
/// depends on the GUID.
#[derive(Debug, Eq, PartialEq)]
#[repr(C)]
pub struct ConfigurationTable {
    pub vendor_guid: Guid,
    pub vendor_table: *mut c_void,
}
