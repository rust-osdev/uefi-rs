// SPDX-License-Identifier: MIT OR Apache-2.0

//! Configuration table utilities.
//!
//! The configuration table is an array of GUIDs and pointers to extra system tables.
//!
//! For example, it can be used to find the ACPI tables.
//!
//! This module contains the actual entries of the configuration table,
//! as well as GUIDs for many known vendor tables.
//!
//! See <https://uefi.org/specs/UEFI/2.10/04_EFI_System_Table.html#efi-configuration-table-properties-table>.

use crate::{guid, Guid};
use bitflags::bitflags;
use core::ffi::c_void;

/// Contains a set of GUID / pointer for a vendor-specific table.
///
/// The UEFI standard guarantees each entry is unique.
///
/// See <https://uefi.org/specs/UEFI/2.10/04_EFI_System_Table.html#efi-configuration-table>.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(C)]
pub struct ConfigTableEntry {
    /// The GUID identifying this table.
    pub guid: Guid,
    /// The starting address of this table.
    ///
    /// Whether this is a physical or virtual address depends on the table.
    pub address: *const c_void,
}

impl ConfigTableEntry {
    /// Entry pointing to the old ACPI 1 RSDP.
    pub const ACPI_GUID: Guid = guid!("eb9d2d30-2d88-11d3-9a16-0090273fc14d");

    /// Entry pointing to the ACPI 2 RSDP.
    pub const ACPI2_GUID: Guid = guid!("8868e871-e4f1-11d3-bc22-0080c73c8881");

    /// Entry pointing to the SMBIOS 1.0 table.
    pub const SMBIOS_GUID: Guid = guid!("eb9d2d31-2d88-11d3-9a16-0090273fc14d");

    /// Entry pointing to the SMBIOS 3.0 table.
    pub const SMBIOS3_GUID: Guid = guid!("f2fd1544-9794-4a2c-992e-e5bbcf20e394");

    /// Entry pointing to the EFI System Resource table (ESRT).
    pub const ESRT_GUID: Guid = guid!("b122a263-3661-4f68-9929-78f8b0d62180");

    /// Hand-off Blocks are used to pass data from the early pre-UEFI environment to the UEFI drivers.
    ///
    /// Most OS loaders or applications should not mess with this.
    pub const HAND_OFF_BLOCK_LIST_GUID: Guid = guid!("7739f24c-93d7-11d4-9a3a-0090273fc14d");

    /// Table used in the early boot environment to record memory ranges.
    pub const MEMORY_TYPE_INFORMATION_GUID: Guid = guid!("4c19049f-4137-4dd3-9c10-8b97a83ffdfa");

    /// Used to identify Hand-off Blocks which store
    /// status codes reported during the pre-UEFI environment.
    pub const MEMORY_STATUS_CODE_RECORD_GUID: Guid = guid!("060cc026-4c0d-4dda-8f41-595fef00a502");

    /// Provides additional information about regions within the run-time memory blocks.
    /// See <https://uefi.org/specs/UEFI/2.10/04_EFI_System_Table.html#efi-memory-attributes-table>.
    pub const MEMORY_ATTRIBUTES_GUID: Guid = guid!("dcfa911d-26eb-469f-a220-38b7dc461220");

    /// Constants used for UEFI signature database variable access.
    /// See <https://uefi.org/specs/UEFI/2.11/32_Secure_Boot_and_Driver_Signing.html#uefi-image-variable-guid-variable-name>.
    pub const IMAGE_SECURITY_DATABASE_GUID: Guid = guid!("d719b2cb-3d3a-4596-a3bc-dad00e67656f");

    /// Table which provides Driver eXecution Environment services.
    pub const DXE_SERVICES_GUID: Guid = guid!("05ad34ba-6f02-4214-952e-4da0398e2bb9");

    /// LZMA-compressed filesystem.
    pub const LZMA_COMPRESS_GUID: Guid = guid!("ee4e5898-3914-4259-9d6e-dc7bd79403cf");

    /// A custom compressed filesystem used by the Tiano UEFI implementation.
    pub const TIANO_COMPRESS_GUID: Guid = guid!("a31280ad-481e-41b6-95e8-127f4c984779");

    /// Pointer to the debug image info table.
    pub const DEBUG_IMAGE_INFO_GUID: Guid = guid!("49152e77-1ada-4764-b7a2-7afefed95e8b");

    /// GUID of the UEFI properties table.
    ///
    /// The properties table is used to provide additional info
    /// about the UEFI implementation.
    pub const PROPERTIES_TABLE_GUID: Guid = guid!("880aaca3-4adc-4a04-9079-b747340825e5");
}

/// Entry pointing to the old ACPI 1 RSDP.
#[deprecated(
    since = "0.35.1",
    note = "please use `ConfigTableEntry::ACPI_GUID` instead"
)]
pub const ACPI_GUID: Guid = ConfigTableEntry::ACPI_GUID;

/// Entry pointing to the ACPI 2 RSDP.
#[deprecated(
    since = "0.35.1",
    note = "please use `ConfigTableEntry::ACPI2_GUID` instead"
)]
pub const ACPI2_GUID: Guid = ConfigTableEntry::ACPI2_GUID;

/// Entry pointing to the SMBIOS 1.0 table.
#[deprecated(
    since = "0.35.1",
    note = "please use `ConfigTableEntry::SMBIOS_GUID` instead"
)]
pub const SMBIOS_GUID: Guid = ConfigTableEntry::SMBIOS_GUID;

/// Entry pointing to the SMBIOS 3.0 table.
#[deprecated(
    since = "0.35.1",
    note = "please use `ConfigTableEntry::SMBIOS3_GUID` instead"
)]
pub const SMBIOS3_GUID: Guid = ConfigTableEntry::SMBIOS3_GUID;

/// Entry pointing to the EFI System Resource table (ESRT).
#[deprecated(
    since = "0.35.1",
    note = "please use `ConfigTableEntry::ESRT_GUID` instead"
)]
pub const ESRT_GUID: Guid = ConfigTableEntry::ESRT_GUID;

/// GUID of the UEFI properties table.
///
/// The properties table is used to provide additional info
/// about the UEFI implementation.
#[deprecated(
    since = "0.35.1",
    note = "please use `ConfigTableEntry::PROPERTIES_TABLE_GUID` instead"
)]
pub const PROPERTIES_TABLE_GUID: Guid = ConfigTableEntry::PROPERTIES_TABLE_GUID;

/// This table contains additional information about the UEFI implementation.
#[repr(C)]
#[derive(Debug)]
pub struct PropertiesTable {
    /// Version of the UEFI properties table.
    ///
    /// The only valid version currently is 0x10_000.
    pub version: u32,
    /// Length in bytes of this table.
    ///
    /// The initial version's length is 16.
    pub length: u32,
    /// Memory protection attributes.
    pub memory_protection: MemoryProtectionAttribute,
}

bitflags! {
    /// Flags describing memory protection.
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct MemoryProtectionAttribute: usize {
        /// If this bit is set, then the UEFI implementation will mark pages
        /// containing data as non-executable.
        const NON_EXECUTABLE_DATA = 1;
    }
}

/// Hand-off Blocks are used to pass data from the early pre-UEFI environment to the UEFI drivers.
///
/// Most OS loaders or applications should not mess with this.
#[deprecated(
    since = "0.35.1",
    note = "please use `ConfigTableEntry::HAND_OFF_BLOCK_LIST_GUID` instead"
)]
pub const HAND_OFF_BLOCK_LIST_GUID: Guid = ConfigTableEntry::HAND_OFF_BLOCK_LIST_GUID;

/// Table used in the early boot environment to record memory ranges.
#[deprecated(
    since = "0.35.1",
    note = "please use `ConfigTableEntry::MEMORY_TYPE_INFORMATION_GUID` instead"
)]
pub const MEMORY_TYPE_INFORMATION_GUID: Guid = ConfigTableEntry::MEMORY_TYPE_INFORMATION_GUID;

/// Used to identify Hand-off Blocks which store
/// status codes reported during the pre-UEFI environment.
#[deprecated(
    since = "0.35.1",
    note = "please use `ConfigTableEntry::MEMORY_STATUS_CODE_RECORD_GUID` instead"
)]
pub const MEMORY_STATUS_CODE_RECORD_GUID: Guid = ConfigTableEntry::MEMORY_STATUS_CODE_RECORD_GUID;

/// Table which provides Driver eXecution Environment services.
#[deprecated(
    since = "0.35.1",
    note = "please use `ConfigTableEntry::DXE_SERVICES_GUID` instead"
)]
pub const DXE_SERVICES_GUID: Guid = ConfigTableEntry::DXE_SERVICES_GUID;

/// LZMA-compressed filesystem.
#[deprecated(
    since = "0.35.1",
    note = "please use `ConfigTableEntry::LZMA_COMPRESS_GUID` instead"
)]
pub const LZMA_COMPRESS_GUID: Guid = ConfigTableEntry::LZMA_COMPRESS_GUID;

/// A custom compressed filesystem used by the Tiano UEFI implementation.
#[deprecated(
    since = "0.35.1",
    note = "please use `ConfigTableEntry::TIANO_COMPRESS_GUID` instead"
)]
pub const TIANO_COMPRESS_GUID: Guid = ConfigTableEntry::TIANO_COMPRESS_GUID;
/// Pointer to the debug image info table.
#[deprecated(
    since = "0.35.1",
    note = "please use `ConfigTableEntry::DEBUG_IMAGE_INFO_GUID` instead"
)]
pub const DEBUG_IMAGE_INFO_GUID: Guid = ConfigTableEntry::DEBUG_IMAGE_INFO_GUID;
