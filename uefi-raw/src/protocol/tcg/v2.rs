// SPDX-License-Identifier: MIT OR Apache-2.0

//! [TCG] (Trusted Computing Group) protocol for [TPM] (Trusted Platform
//! Module) 2.0.
//!
//! This protocol is defined in the [TCG EFI Protocol Specification _TPM
//! Family 2.0_][spec]. It is generally implemented only for TPM 2.0
//! devices, but the spec indicates it can also be used for older TPM
//! devices.
//!
//! [spec]: https://trustedcomputinggroup.org/resource/tcg-efi-protocol-specification/
//! [TCG]: https://trustedcomputinggroup.org/
//! [TPM]: https://en.wikipedia.org/wiki/Trusted_Platform_Module

use super::EventType;
use crate::{guid, Guid, PhysicalAddress, Status};
use bitflags::bitflags;
use core::ffi::c_void;

/// Version information.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Tcg2Version {
    /// Major version.
    pub major: u8,
    /// Minor version.
    pub minor: u8,
}

bitflags! {
    /// Event log formats supported by the firmware.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(transparent)]
    pub struct Tcg2EventLogBitmap: u32 {
        /// Firmware supports the SHA-1 log format.
        const TCG_1_2 = 0x0000_0001;

        /// Firmware supports the crypto-agile log format.
        const TCG_2 = 0x0000_0002;
    }
}

/// Event log formats supported by the firmware.
pub type Tcg2EventLogFormat = Tcg2EventLogBitmap;

bitflags! {
    /// Hash algorithms the protocol can provide.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(transparent)]
    pub struct Tcg2HashAlgorithmBitmap: u32 {
        /// SHA-1 hash.
        const SHA1 = 0x0000_0001;

        /// SHA-256 hash.
        const SHA256 = 0x0000_0002;

        /// SHA-384 hash.
        const SHA384 = 0x0000_0004;

        /// SHA-512 hash.
        const SHA512 = 0x0000_0008;

        /// SM3-256 hash.
        const SM3_256 = 0x0000_0010;
    }
}

/// Information about the protocol and the TPM device.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Tcg2BootServiceCapability {
    /// Size of this structure.
    pub size: u8,

    /// Version of the EFI TCG2 protocol.
    pub structure_version: Tcg2Version,

    /// Version of the EFI TCG2 protocol.
    pub protocol_version: Tcg2Version,

    /// Bitmap of supported hash algorithms.
    pub hash_algorithm_bitmap: Tcg2HashAlgorithmBitmap,

    /// Event log formats supported by the firmware.
    pub supported_event_logs: Tcg2EventLogBitmap,

    /// Whether the TPM is present or not.
    pub tpm_present_flag: u8,

    /// Maximum size (in bytes) of a command that can be sent to the TPM.
    pub max_command_size: u16,

    /// Maximum size (in bytes) of a response that can be provided by the TPM.
    pub max_response_size: u16,

    /// Manufacturer ID.
    ///
    /// See the [TCG Vendor ID registry].
    ///
    /// [TCG Vendor ID registry]: https://trustedcomputinggroup.org/resource/vendor-id-registry/
    pub manufacturer_id: u32,

    /// Maximum number of supported PCR banks (hashing algorithms).
    pub number_of_pcr_banks: u32,

    /// Bitmap of currently-active PCR banks (hashing algorithms). This
    /// is a subset of the supported algorithms in [`hash_algorithm_bitmap`].
    ///
    /// [`hash_algorithm_bitmap`]: Self::hash_algorithm_bitmap
    pub active_pcr_banks: Tcg2HashAlgorithmBitmap,
}

bitflags! {
    /// Flags for the [`Tcg::hash_log_extend_event`] function.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(transparent)]
    pub struct Tcg2HashLogExtendEventFlags: u64 {
        /// Extend an event but don't log it.
        const EFI_TCG2_EXTEND_ONLY = 0x0000_0000_0000_0001;

        /// Use when measuring a PE/COFF image.
        const PE_COFF_IMAGE = 0x0000_0000_0000_0010;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C, packed)]
pub struct Tcg2EventHeader {
    pub header_size: u32,
    pub header_version: u16,
    pub pcr_index: u32,
    pub event_type: EventType,
}

/// Protocol for interacting with TPM devices.
///
/// This protocol can be used for interacting with older TPM 1.1/1.2
/// devices, but most firmware only uses it for TPM 2.0.
///
/// The corresponding C type is `EFI_TCG2_PROTOCOL`.
#[derive(Debug)]
#[repr(C)]
pub struct Tcg2Protocol {
    pub get_capability: unsafe extern "efiapi" fn(
        this: *mut Self,
        protocol_capability: *mut Tcg2BootServiceCapability,
    ) -> Status,

    pub get_event_log: unsafe extern "efiapi" fn(
        this: *mut Self,
        event_log_format: Tcg2EventLogFormat,
        event_log_location: *mut PhysicalAddress,
        event_log_last_entry: *mut PhysicalAddress,
        event_log_truncated: *mut u8,
    ) -> Status,

    pub hash_log_extend_event: unsafe extern "efiapi" fn(
        this: *mut Self,
        flags: Tcg2HashLogExtendEventFlags,
        data_to_hash: PhysicalAddress,
        data_to_hash_len: u64,
        event: *const c_void,
    ) -> Status,

    pub submit_command: unsafe extern "efiapi" fn(
        this: *mut Self,
        input_parameter_block_size: u32,
        input_parameter_block: *const u8,
        output_parameter_block_size: u32,
        output_parameter_block: *mut u8,
    ) -> Status,

    pub get_active_pcr_banks: unsafe extern "efiapi" fn(
        this: *mut Self,
        active_pcr_banks: *mut Tcg2HashAlgorithmBitmap,
    ) -> Status,

    pub set_active_pcr_banks: unsafe extern "efiapi" fn(
        this: *mut Self,
        active_pcr_banks: Tcg2HashAlgorithmBitmap,
    ) -> Status,

    pub get_result_of_set_active_pcr_banks: unsafe extern "efiapi" fn(
        this: *mut Self,
        operation_present: *mut u32,
        response: *mut u32,
    ) -> Status,
}

impl Tcg2Protocol {
    pub const GUID: Guid = guid!("607f766c-7455-42be-930b-e4d76db2720f");
}
