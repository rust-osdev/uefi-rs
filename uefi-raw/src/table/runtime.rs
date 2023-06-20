//! UEFI services available at runtime, even after the OS boots.

use crate::capsule::CapsuleHeader;
use crate::table::boot::MemoryDescriptor;
use crate::table::Header;
use crate::time::Time;
use crate::{guid, Char16, Guid, PhysicalAddress, Status};
use bitflags::bitflags;
use core::ffi::c_void;

/// Table of pointers to all the runtime services.
///
/// This table, and the function pointers it contains are valid even after the
/// UEFI OS loader and OS have taken control of the platform.
#[repr(C)]
pub struct RuntimeServices {
    pub header: Header,
    pub get_time:
        unsafe extern "efiapi" fn(time: *mut Time, capabilities: *mut TimeCapabilities) -> Status,
    pub set_time: unsafe extern "efiapi" fn(time: *const Time) -> Status,
    pub get_wakeup_time:
        unsafe extern "efiapi" fn(enabled: *mut u8, pending: *mut u8, time: *mut Time) -> Status,
    pub set_wakeup_time: unsafe extern "efiapi" fn(enable: u8, time: *const Time) -> Status,
    pub set_virtual_address_map: unsafe extern "efiapi" fn(
        map_size: usize,
        desc_size: usize,
        desc_version: u32,
        virtual_map: *mut MemoryDescriptor,
    ) -> Status,
    pub convert_pointer:
        unsafe extern "efiapi" fn(debug_disposition: usize, address: *mut *const c_void) -> Status,
    pub get_variable: unsafe extern "efiapi" fn(
        variable_name: *const Char16,
        vendor_guid: *const Guid,
        attributes: *mut VariableAttributes,
        data_size: *mut usize,
        data: *mut u8,
    ) -> Status,
    pub get_next_variable_name: unsafe extern "efiapi" fn(
        variable_name_size: *mut usize,
        variable_name: *mut u16,
        vendor_guid: *mut Guid,
    ) -> Status,
    pub set_variable: unsafe extern "efiapi" fn(
        variable_name: *const Char16,
        vendor_guid: *const Guid,
        attributes: VariableAttributes,
        data_size: usize,
        data: *const u8,
    ) -> Status,
    pub get_next_high_monotonic_count: unsafe extern "efiapi" fn(high_count: *mut u32) -> Status,
    pub reset_system: unsafe extern "efiapi" fn(
        rt: ResetType,
        status: Status,
        data_size: usize,
        data: *const u8,
    ) -> !,

    // UEFI 2.0 Capsule Services.
    pub update_capsule: unsafe extern "efiapi" fn(
        capsule_header_array: *const *const CapsuleHeader,
        capsule_count: usize,
        scatter_gather_list: PhysicalAddress,
    ) -> Status,
    pub query_capsule_capabilities: unsafe extern "efiapi" fn(
        capsule_header_array: *const *const CapsuleHeader,
        capsule_count: usize,
        maximum_capsule_size: *mut usize,
        reset_type: *mut ResetType,
    ) -> Status,

    // Miscellaneous UEFI 2.0 Service.
    pub query_variable_info: unsafe extern "efiapi" fn(
        attributes: VariableAttributes,
        maximum_variable_storage_size: *mut u64,
        remaining_variable_storage_size: *mut u64,
        maximum_variable_size: *mut u64,
    ) -> Status,
}

newtype_enum! {
    /// The type of system reset.
    pub enum ResetType: u32 => {
        /// System-wide reset.
        ///
        /// This is analogous to power cycling the device.
        COLD = 0,

        /// System-wide re-initialization.
        ///
        /// If the system doesn't support a warm reset, this will trigger a cold
        /// reset.
        WARM = 1,

        /// The system is powered off.
        SHUTDOWN = 2,

        /// A platform-specific reset type.
        PLATFORM_SPECIFIC = 3,
    }
}

/// Real time clock capabilities.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct TimeCapabilities {
    /// Reporting resolution of the clock in counts per second. 1 for a normal
    /// PC-AT CMOS RTC device, which reports the time with 1-second resolution.
    pub resolution: u32,

    /// Timekeeping accuracy in units of 1e-6 parts per million.
    pub accuracy: u32,

    /// Whether a time set operation clears the device's time below the
    /// "resolution" reporting level. False for normal PC-AT CMOS RTC devices.
    pub sets_to_zero: bool,
}

bitflags! {
    /// Flags describing the attributes of a variable.
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct VariableAttributes: u32 {
        /// Variable is maintained across a power cycle.
        const NON_VOLATILE = 0x01;

        /// Variable is accessible during the time that boot services are
        /// accessible.
        const BOOTSERVICE_ACCESS = 0x02;

        /// Variable is accessible during the time that runtime services are
        /// accessible.
        const RUNTIME_ACCESS = 0x04;

        /// Variable is stored in the portion of NVR allocated for error
        /// records.
        const HARDWARE_ERROR_RECORD = 0x08;

        /// Deprecated.
        const AUTHENTICATED_WRITE_ACCESS = 0x10;

        /// Variable payload begins with an EFI_VARIABLE_AUTHENTICATION_2
        /// structure.
        const TIME_BASED_AUTHENTICATED_WRITE_ACCESS = 0x20;

        /// This is never set in the attributes returned by
        /// `get_variable`. When passed to `set_variable`, the variable payload
        /// will be appended to the current value of the variable if supported
        /// by the firmware.
        const APPEND_WRITE = 0x40;

        /// Variable payload begins with an EFI_VARIABLE_AUTHENTICATION_3
        /// structure.
        const ENHANCED_AUTHENTICATED_ACCESS = 0x80;
    }
}

newtype_enum! {
    /// Variable vendor GUID. This serves as a namespace for variables to
    /// avoid naming conflicts between vendors. The UEFI specification
    /// defines some special values, and vendors will define their own.
    pub enum VariableVendor: Guid => {
        /// Used to access global variables.
        GLOBAL_VARIABLE = guid!("8be4df61-93ca-11d2-aa0d-00e098032b8c"),

        /// Used to access EFI signature database variables.
        IMAGE_SECURITY_DATABASE = guid!("d719b2cb-3d3a-4596-a3bc-dad00e67656f"),
    }
}
