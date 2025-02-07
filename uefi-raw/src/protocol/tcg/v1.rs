// SPDX-License-Identifier: MIT OR Apache-2.0

//! [TCG] (Trusted Computing Group) protocol for [TPM] (Trusted Platform
//! Module) 1.1 and 1.2.
//!
//! This protocol is defined in the [TCG EFI Protocol Specification _for
//! TPM Family 1.1 or 1.2_][spec].
//!
//! [spec]: https://trustedcomputinggroup.org/resource/tcg-efi-protocol-specification/
//! [TCG]: https://trustedcomputinggroup.org/
//! [TPM]: https://en.wikipedia.org/wiki/Trusted_Platform_Module

use crate::{guid, Guid, PhysicalAddress, Status};
use core::ffi::c_void;

/// Information about the protocol and the TPM device.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct TcgBootServiceCapability {
    pub size: u8,
    pub structure_version: TcgVersion,
    pub protocol_spec_version: TcgVersion,
    pub hash_algorithm_bitmap: u8,
    pub tpm_present_flag: u8,
    pub tpm_deactivated_flag: u8,
}

/// Version information.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct TcgVersion {
    /// Major version.
    pub major: u8,
    /// Minor version.
    pub minor: u8,

    pub rev_major: u8,
    pub rev_minor: u8,
}

/// Protocol for interacting with TPM 1.1 and 1.2 devices.
#[derive(Debug)]
#[repr(C)]
pub struct TcgProtocol {
    pub status_check: unsafe extern "efiapi" fn(
        this: *mut Self,
        protocol_capability: *mut TcgBootServiceCapability,
        feature_flags: *mut u32,
        event_log_location: *mut PhysicalAddress,
        event_log_last_entry: *mut PhysicalAddress,
    ) -> Status,

    pub hash_all: unsafe extern "efiapi" fn(
        this: *mut Self,
        hash_data: *const u8,
        hash_data_len: u64,
        algorithm_id: u32,
        hashed_data_len: *mut u64,
        hashed_data_result: *mut *mut u8,
    ) -> Status,

    pub log_event: unsafe extern "efiapi" fn(
        this: *mut Self,
        event: *const c_void,
        event_number: *mut u32,
        flags: u32,
    ) -> Status,

    pub pass_through_to_tpm: unsafe extern "efiapi" fn(
        this: *mut Self,
        tpm_input_parameter_block_size: u32,
        tpm_input_parameter_block: *const u8,
        tpm_output_parameter_block_size: u32,
        tpm_output_parameter_block: *mut u8,
    ) -> Status,

    pub hash_log_extend_event: unsafe extern "efiapi" fn(
        this: *mut Self,
        hash_data: PhysicalAddress,
        hash_data_len: u64,
        algorithm_id: u32,
        event: *mut c_void,
        event_number: *mut u32,
        event_log_last_entry: *mut PhysicalAddress,
    ) -> Status,
}

impl TcgProtocol {
    pub const GUID: Guid = guid!("f541796d-a62e-4954-a775-9584f61b9cdd");
}
