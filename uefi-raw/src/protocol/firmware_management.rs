// SPDX-License-Identifier: MIT OR Apache-2.0

//! Firmware update and reporting

use crate::{Char16, Guid, Status, guid, newtype_enum};
use core::ffi::c_void;

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[repr(transparent)]
    pub struct CapsuleSupport: u64 {
        const AUTHENTICATION = 1 << 0;
        const DEPENDENCY = 1 << 1;
    }
}

/// EFI_FIRMWARE_MANAGEMENT_CAPSULE_HEADER
#[derive(Debug)]
#[repr(C, packed)]
pub struct FirmwareManagementCapsuleHeader {
    pub version: u32,
    pub embedded_driver_count: u16,
    pub payload_item_count: u16,
    pub item_offset_list: [u64; 0],
}

impl FirmwareManagementCapsuleHeader {
    pub const INIT_VERSION: u32 = 1;
}

/// EFI_FIRMWARE_MANAGEMENT_CAPSULE_IMAGE_HEADER
#[derive(Debug)]
#[repr(C, packed)]
pub struct FirmwareManagementCapsuleImageHeader {
    pub version: u32,
    pub update_image_type_id: Guid,
    pub update_image_index: u8,
    pub reserved_bytes: [u8; 3],
    pub update_image_size: u32,
    pub update_vendor_code_size: u32,
    pub update_hardware_instance: u64,
    pub image_capsule_support: CapsuleSupport,
}

impl FirmwareManagementCapsuleImageHeader {
    pub const INIT_VERSION: u32 = 3;
}

newtype_enum! {
    /// FMP dependency expression opcodes (`EFI_FMP_DEP_*`)
    pub enum FmpDep: u8 => {
        PUSH_GUID = 0x00,
        PUSH_VERSION = 0x01,
        VERSION_STR = 0x02,
        AND = 0x03,
        OR = 0x04,
        NOT = 0x05,
        TRUE = 0x06,
        FALSE = 0x07,
        EQ = 0x08,
        GT = 0x09,
        GTE = 0x0A,
        LT = 0x0B,
        LTE = 0x0C,
        END = 0x0D,
        DECLARE_LENGTH = 0x0E,
    }
}

/// EFI_FIRMWARE_IMAGE_DEP
#[derive(Debug)]
#[repr(C)]
pub struct FirmwareImageDep {
    pub dependencies: [u8; 0],
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[repr(transparent)]
    pub struct ImageAttributes: u64 {
        const IMAGE_UPDATABLE = 1 << 0;
        const RESET_REQUIRED = 1 << 1;
        const AUTHENTICATION_REQUIRED = 1 << 2;
        const IN_USE = 1 << 3;
        const UEFI_IMAGE = 1 << 4;
        const DEPENDENCY = 1 << 5;
    }
}

bitflags::bitflags! {
    // Lower 16 bits are reserved for UEFI assignment.
    // Other bits are for vendor-specific compatibility checks.
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[repr(transparent)]
    pub struct ImageCompatibilities: u64 {
        const CHECK_SUPPORTED = 1 << 0;
        const _ = !0;
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[repr(transparent)]
    pub struct ImageUpdatable: u32 {
        const VALID = 1 << 0;
        const INVALID = 1 << 1;
        const INVALID_TYPE = 1 << 2;
        const INVALID_OLD = 1  << 3;
        const VALID_WITH_VENDOR_CODE = 1 << 4;
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[repr(transparent)]
    pub struct PackageAttributes: u64 {
        const UPDATABLE = 1 << 0;
        const RESET_REQUIRED = 1 << 1;
        const AUTHENTICATION_REQUIRED = 1 << 2;
    }
}

/// EFI_FIRMWARE_IMAGE_DESCRIPTOR
#[derive(Debug)]
#[repr(C)]
pub struct FirmwareImageDescriptor {
    pub image_index: u8,
    pub image_type_id: Guid,
    pub image_id: u64,
    pub image_id_name: *const Char16,
    pub version: u32,
    pub version_name: *const Char16,
    pub size: usize,
    pub attributes_supported: ImageAttributes,
    pub attributes_setting: ImageAttributes,
    pub compatibilities: ImageCompatibilities,
    pub lowest_supported_image_version: u32,
    pub last_attempt_version: u32,
    pub last_attempt_status: u32,
    pub hardware_instance: u64,
    pub dependencies: *const FirmwareImageDep,
}

impl FirmwareImageDescriptor {
    pub const VERSION: u32 = 4;
}

/// EFI_FIRMWARE_MANAGEMENT_PROTOCOL
#[derive(Debug)]
#[repr(C)]
pub struct FirmwareManagementProtocol {
    pub get_image_info: unsafe extern "efiapi" fn(
        this: *const Self,
        image_info_size: *mut usize,
        image_info: *mut FirmwareImageDescriptor,
        descriptor_version: *mut u32,
        descriptor_count: *mut u8,
        descriptor_size: *mut usize,
        package_version: *mut u32,
        package_version_name: *mut *mut Char16,
    ) -> Status,
    pub get_image: unsafe extern "efiapi" fn(
        this: *const Self,
        image_index: u8,
        image: *mut c_void,
        image_size: *mut usize,
    ) -> Status,
    pub set_image: unsafe extern "efiapi" fn(
        this: *const Self,
        image_index: u8,
        image: *const c_void,
        image_size: usize,
        vendor_code: *const c_void,
        progress: unsafe extern "efiapi" fn(completion: usize) -> Status,
        abort_reason: *mut *mut Char16,
    ) -> Status,
    pub check_image: unsafe extern "efiapi" fn(
        this: *const Self,
        image_index: u8,
        image: *const c_void,
        image_size: usize,
        image_updatable: *mut ImageUpdatable,
    ) -> Status,
    pub get_package_info: unsafe extern "efiapi" fn(
        this: *const Self,
        package_version: *mut u32,
        package_version_name: *mut *mut Char16,
        package_version_name_max_len: *mut u32,
        attributes_supported: *mut PackageAttributes,
        attributes_setting: *mut PackageAttributes,
    ) -> Status,
    pub set_package_info: unsafe extern "efiapi" fn(
        this: *const Self,
        image: *const c_void,
        image_size: usize,
        vendor_code: *const c_void,
        package_version: u32,
        package_version_name: *const Char16,
    ) -> Status,
}

impl FirmwareManagementProtocol {
    pub const GUID: Guid = guid!("86c77a67-0b97-4633-a187-49104d0685c7");
}
