//! [TCG] (Trusted Computing Group) protocol for [TPM] (Trusted Platform
//! Module) 1.1 and 1.2.
//!
//! This protocol is defined in the [TCG EFI Protocol Specification _for
//! TPM Family 1.1 or 1.2_][spec].
//!
//! [spec]: https://trustedcomputinggroup.org/resource/tcg-efi-protocol-specification/
//! [TCG]: https://trustedcomputinggroup.org/
//! [TPM]: https://en.wikipedia.org/wiki/Trusted_Platform_Module

use super::{AlgorithmId, EventType, HashAlgorithm, PcrIndex};
use crate::data_types::PhysicalAddress;
use crate::polyfill::maybe_uninit_slice_as_mut_ptr;
use crate::proto::unsafe_protocol;
use crate::util::{ptr_write_unaligned_and_add, usize_from_u32};
use crate::{Error, Result, Status, StatusExt};
use core::fmt::{self, Debug, Formatter};
use core::marker::PhantomData;
use core::mem::{self, MaybeUninit};
use core::ptr;
use ptr_meta::Pointee;

/// 20-byte SHA-1 digest.
pub type Sha1Digest = [u8; 20];

/// This corresponds to the `AlgorithmId` enum, but in the v1 spec it's `u32`
/// instead of `u16`.
#[allow(non_camel_case_types)]
type TCG_ALGORITHM_ID = u32;

/// Information about the protocol and the TPM device.
///
/// Layout compatible with the C type `TCG_EFI_BOOT_SERVICE_CAPABILITY`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct BootServiceCapability {
    size: u8,
    structure_version: Version,
    protocol_spec_version: Version,
    hash_algorithm_bitmap: u8,
    tpm_present_flag: u8,
    tpm_deactivated_flag: u8,
}

impl BootServiceCapability {
    /// Version of the `BootServiceCapability` structure.
    #[must_use]
    pub fn structure_version(&self) -> Version {
        self.structure_version
    }

    /// Version of the `Tcg` protocol.
    #[must_use]
    pub fn protocol_spec_version(&self) -> Version {
        self.protocol_spec_version
    }

    /// Supported hash algorithms.
    #[must_use]
    pub fn hash_algorithm(&self) -> HashAlgorithm {
        // Safety: the value should always be 0x1 (indicating SHA-1), but
        // we don't care if it's some unexpected value.
        HashAlgorithm::from_bits_retain(u32::from(self.hash_algorithm_bitmap))
    }

    /// Whether the TPM device is present.
    #[must_use]
    pub fn tpm_present(&self) -> bool {
        self.tpm_present_flag != 0
    }

    /// Whether the TPM device is deactivated.
    #[must_use]
    pub fn tpm_deactivated(&self) -> bool {
        self.tpm_deactivated_flag != 0
    }
}

/// Version information.
///
/// Layout compatible with the C type `TCG_VERSION`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Version {
    /// Major version.
    pub major: u8,
    /// Minor version.
    pub minor: u8,

    // Leave these two fields undocumented since it's not clear what
    // they are for. The spec doesn't say, and they were removed in the
    // v2 spec.
    #[allow(missing_docs)]
    pub rev_major: u8,
    #[allow(missing_docs)]
    pub rev_minor: u8,
}

/// Entry in the [`EventLog`].
///
/// Layout compatible with the C type `TCG_PCR_EVENT`.
///
/// Naming note: the spec refers to "event data" in two conflicting
/// ways: the `event_data` field and the data hashed in the digest
/// field. These two are independent; although the event data _can_ be
/// what is hashed in the digest field, it doesn't have to be.
#[repr(C, packed)]
#[derive(Eq, Pointee)]
pub struct PcrEvent {
    pcr_index: PcrIndex,
    event_type: EventType,
    digest: Sha1Digest,
    event_data_size: u32,
    event_data: [u8],
}

impl PcrEvent {
    pub(super) unsafe fn from_ptr<'a>(ptr: *const u8) -> &'a Self {
        // Get the `event_size` field.
        let ptr_u32: *const u32 = ptr.cast();
        let event_size = ptr_u32.add(7).read_unaligned();
        let event_size = usize_from_u32(event_size);
        unsafe { &*ptr_meta::from_raw_parts(ptr.cast(), event_size) }
    }

    /// Create a new `PcrEvent` using a byte buffer for storage.
    ///
    /// # Errors
    ///
    /// Returns [`Status::BUFFER_TOO_SMALL`] if the `buffer` is not large
    /// enough.
    ///
    /// Returns [`Status::INVALID_PARAMETER`] if the `event_data` size is too
    /// large.
    pub fn new_in_buffer<'buf>(
        buffer: &'buf mut [MaybeUninit<u8>],
        pcr_index: PcrIndex,
        event_type: EventType,
        digest: Sha1Digest,
        event_data: &[u8],
    ) -> Result<&'buf mut Self> {
        let event_data_size =
            u32::try_from(event_data.len()).map_err(|_| Error::from(Status::INVALID_PARAMETER))?;

        let required_size = mem::size_of::<PcrIndex>()
            + mem::size_of::<EventType>()
            + mem::size_of::<Sha1Digest>()
            + mem::size_of::<u32>()
            + event_data.len();

        if buffer.len() < required_size {
            return Err(Status::BUFFER_TOO_SMALL.into());
        }

        let mut ptr: *mut u8 = maybe_uninit_slice_as_mut_ptr(buffer);

        unsafe {
            ptr_write_unaligned_and_add(&mut ptr, pcr_index);
            ptr_write_unaligned_and_add(&mut ptr, event_type);
            ptr_write_unaligned_and_add(&mut ptr, digest);
            ptr_write_unaligned_and_add(&mut ptr, event_data_size);
            ptr::copy(event_data.as_ptr(), ptr, event_data.len());

            let ptr: *mut PcrEvent =
                ptr_meta::from_raw_parts_mut(buffer.as_mut_ptr().cast(), event_data.len());
            Ok(&mut *ptr)
        }
    }

    /// PCR index for the event.
    #[must_use]
    pub fn pcr_index(&self) -> PcrIndex {
        self.pcr_index
    }

    /// Type of event, indicating what type of data is stored in [`event_data`].
    ///
    /// [`event_data`]: Self::event_data
    #[must_use]
    pub fn event_type(&self) -> EventType {
        self.event_type
    }

    /// Raw event data. The meaning of this data can be determined from
    /// the [`event_type`].
    ///
    /// Note that this data is independent of what is hashed [`digest`].
    ///
    /// [`digest`]: Self::digest
    /// [`event_type`]: Self::event_type
    #[must_use]
    pub fn event_data(&self) -> &[u8] {
        &self.event_data
    }

    /// SHA-1 digest of the data hashed for this event.
    #[must_use]
    pub fn digest(&self) -> Sha1Digest {
        self.digest
    }
}

// Manual `Debug` implementation since it can't be derived for a packed DST.
impl Debug for PcrEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PcrEvent")
            .field("pcr_index", &{ self.pcr_index })
            .field("event_type", &{ self.event_type })
            .field("digest", &self.digest)
            .field("event_data_size", &{ self.event_data_size })
            .field("event_data", &&self.event_data)
            .finish()
    }
}

// Manual `PartialEq` implementation since it can't be derived for a packed DST.
impl PartialEq for PcrEvent {
    fn eq(&self, rhs: &Self) -> bool {
        self.pcr_index() == rhs.pcr_index()
            && self.event_type() == rhs.event_type()
            && self.digest == rhs.digest
            && self.event_data_size == rhs.event_data_size
            && self.event_data == rhs.event_data
    }
}

opaque_type! {
    /// Opaque type that should be used to represent a pointer to a [`PcrEvent`] in
    /// foreign function interfaces. This type produces a thin pointer, unlike
    /// [`PcrEvent`].
    pub struct FfiPcrEvent;
}

/// TPM event log.
///
/// This type of event log always uses SHA-1 hashes. The [`v1::Tcg`]
/// protocol always uses this type of event log, but it can also be
/// provided by the [`v2::Tcg`] protocol via [`get_event_log_v2`].
///
/// [`v1::Tcg`]: Tcg
/// [`v2::Tcg`]: super::v2::Tcg
/// [`get_event_log_v2`]: super::v2::Tcg::get_event_log_v2
#[derive(Debug)]
pub struct EventLog<'a> {
    // Tie the lifetime to the protocol, and by extension, boot services.
    _lifetime: PhantomData<&'a Tcg>,

    location: *const u8,
    last_entry: *const u8,

    is_truncated: bool,
}

impl<'a> EventLog<'a> {
    pub(super) unsafe fn new(
        location: *const u8,
        last_entry: *const u8,
        is_truncated: bool,
    ) -> Self {
        Self {
            _lifetime: PhantomData,
            location,
            last_entry,
            is_truncated,
        }
    }

    /// Iterator of events in the log.
    #[must_use]
    pub fn iter(&self) -> EventLogIter {
        EventLogIter {
            log: self,
            location: self.location,
        }
    }

    /// If true, the event log is missing one or more entries because
    /// additional events would have exceeded the space allocated for
    /// the log.
    ///
    /// This value is not reported for the [`v1::Tcg`] protocol, so it
    /// is always `false` in that case.
    ///
    /// [`v1::Tcg`]: Tcg
    #[must_use]
    pub fn is_truncated(&self) -> bool {
        self.is_truncated
    }
}

/// Iterator for events in [`EventLog`].
#[derive(Debug)]
pub struct EventLogIter<'a> {
    log: &'a EventLog<'a>,
    location: *const u8,
}

impl<'a> Iterator for EventLogIter<'a> {
    type Item = &'a PcrEvent;

    fn next(&mut self) -> Option<Self::Item> {
        // The spec says that `last_entry` will be null if there are no
        // events. Presumably `location` will be null as well, but check
        // both just to be safe.
        if self.location.is_null() || self.log.last_entry.is_null() {
            return None;
        }

        // Safety: we trust that the protocol has given us a valid range
        // of memory to read from.
        let event = unsafe { PcrEvent::from_ptr(self.location) };

        // If this is the last entry, set the location to null so that
        // future calls to `next()` return `None`.
        if self.location == self.log.last_entry {
            self.location = ptr::null();
        } else {
            self.location = unsafe { self.location.add(mem::size_of_val(event)) };
        }

        Some(event)
    }
}

/// Protocol for interacting with TPM 1.1 and 1.2 devices.
///
/// The corresponding C type is `EFI_TCG_PROTOCOL`.
#[repr(C)]
#[unsafe_protocol("f541796d-a62e-4954-a775-9584f61b9cdd")]
pub struct Tcg {
    status_check: unsafe extern "efiapi" fn(
        this: *mut Tcg,
        protocol_capability: *mut BootServiceCapability,
        feature_flags: *mut u32,
        event_log_location: *mut PhysicalAddress,
        event_log_last_entry: *mut PhysicalAddress,
    ) -> Status,

    // Note: we do not currently expose this function because the spec
    // for this is not well written. The function allocates memory, but
    // the spec doesn't say how to free it. Most likely
    // `EFI_BOOT_SERVICES.FreePool` would work, but this is not
    // mentioned in the spec so it is unsafe to rely on.
    //
    // Also, this function is not that useful in practice for a couple
    // reasons. First, it takes an algorithm ID, but only SHA-1 is
    // supported with TPM v1. Second, TPMs are not cryptographic
    // accelerators, so it is very likely faster to calculate the hash
    // on the CPU, e.g. with the `sha1` crate.
    hash_all: unsafe extern "efiapi" fn() -> Status,

    log_event: unsafe extern "efiapi" fn(
        this: *mut Tcg,
        // The spec does not guarantee that the `event` will not be mutated
        // through the pointer, but it seems reasonable to assume and makes the
        // public interface clearer, so use a const pointer.
        event: *const FfiPcrEvent,
        event_number: *mut u32,
        flags: u32,
    ) -> Status,

    pass_through_to_tpm: unsafe extern "efiapi" fn(
        this: *mut Tcg,
        tpm_input_parameter_block_size: u32,
        tpm_input_parameter_block: *const u8,
        tpm_output_parameter_block_size: u32,
        tpm_output_parameter_block: *mut u8,
    ) -> Status,

    hash_log_extend_event: unsafe extern "efiapi" fn(
        this: *mut Tcg,
        hash_data: PhysicalAddress,
        hash_data_len: u64,
        algorithm_id: TCG_ALGORITHM_ID,
        event: *mut FfiPcrEvent,
        event_number: *mut u32,
        event_log_last_entry: *mut PhysicalAddress,
    ) -> Status,
}

/// Return type of [`Tcg::status_check`].
#[derive(Debug)]
pub struct StatusCheck<'a> {
    /// Information about the protocol and the TPM device.
    pub protocol_capability: BootServiceCapability,

    /// Feature flags. The spec does not define any feature flags, so
    /// this is always expected to be zero.
    pub feature_flags: u32,

    /// TPM event log.
    pub event_log: EventLog<'a>,
}

impl Tcg {
    /// Get information about the protocol and TPM device, as well as
    /// the TPM event log.
    pub fn status_check(&mut self) -> Result<StatusCheck> {
        let mut protocol_capability = BootServiceCapability::default();
        let mut feature_flags = 0;
        let mut event_log_location = 0;
        let mut event_log_last_entry = 0;

        let status = unsafe {
            (self.status_check)(
                self,
                &mut protocol_capability,
                &mut feature_flags,
                &mut event_log_location,
                &mut event_log_last_entry,
            )
        };

        if status.is_success() {
            // The truncated field is just there for the v2 protocol;
            // always set it to false for v1.
            let truncated = false;
            let event_log = unsafe {
                EventLog::new(
                    event_log_location as *const u8,
                    event_log_last_entry as *const u8,
                    truncated,
                )
            };

            Ok(StatusCheck {
                protocol_capability,
                feature_flags,
                event_log,
            })
        } else {
            Err(status.into())
        }
    }

    /// Add an entry to the event log without extending a PCR.
    ///
    /// Usually [`hash_log_extend_event`] should be used instead. An
    /// entry added via `log_event` cannot be verified, so it is mainly
    /// intended for adding an informational entry.
    ///
    /// [`hash_log_extend_event`]: Self::hash_log_extend_event
    pub fn log_event(&mut self, event: &PcrEvent) -> Result {
        // This is the only valid value; it indicates that the extend
        // operation should not be performed.
        let flags = 0x1;

        // Don't bother returning this, it's not very useful info.
        let mut event_number = 0;

        let event_ptr: *const PcrEvent = event;

        unsafe { (self.log_event)(self, event_ptr.cast(), &mut event_number, flags).to_result() }
    }

    /// Extend a PCR and add an entry to the event log.
    ///
    /// If `data_to_hash` is `None` then the `digest` field of the `event`
    /// should be used as-is. Otherwise, the `digest` field will be overwritten
    /// with the SHA-1 hash of the data.
    pub fn hash_log_extend_event(
        &mut self,
        event: &mut PcrEvent,
        data_to_hash: Option<&[u8]>,
    ) -> Result {
        let hash_data;
        let hash_data_len;
        if let Some(data_to_hash) = data_to_hash {
            hash_data = data_to_hash.as_ptr() as PhysicalAddress;
            hash_data_len = u64::try_from(data_to_hash.len()).unwrap();
        } else {
            hash_data = 0;
            hash_data_len = 0;
        }

        // Don't bother returning these, it's not very useful info.
        let mut event_number = 0;
        let mut event_log_last_entry = 0;

        let event_ptr: *mut PcrEvent = event;

        unsafe {
            (self.hash_log_extend_event)(
                self,
                hash_data,
                hash_data_len,
                AlgorithmId::SHA1.0.into(),
                event_ptr.cast(),
                &mut event_number,
                &mut event_log_last_entry,
            )
            .to_result()
        }
    }

    /// Send a command directly to the TPM.
    ///
    /// Constructing the input block and parsing the output block are outside
    /// the scope of this crate. See the [TPM 1.2 Main Specification][spec]
    /// documents for details of these blocks, in particular Part 3, Commands.
    ///
    /// Note that TPM structures are big endian.
    ///
    /// [spec]: https://trustedcomputinggroup.org/resource/tpm-main-specification/
    pub fn pass_through_to_tpm(
        &mut self,
        input_parameter_block: &[u8],
        output_parameter_block: &mut [u8],
    ) -> Result {
        let input_parameter_block_len = u32::try_from(input_parameter_block.len())
            .map_err(|_| Error::from(Status::BAD_BUFFER_SIZE))?;
        let output_parameter_block_len = u32::try_from(output_parameter_block.len())
            .map_err(|_| Error::from(Status::BAD_BUFFER_SIZE))?;

        unsafe {
            (self.pass_through_to_tpm)(
                self,
                input_parameter_block_len,
                input_parameter_block.as_ptr(),
                output_parameter_block_len,
                output_parameter_block.as_mut_ptr(),
            )
            .to_result()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::slice;

    #[test]
    fn test_new_pcr_event() {
        let mut event_buf = [MaybeUninit::uninit(); 256];
        #[rustfmt::skip]
        let digest = [
            0x00, 0x01, 0x02, 0x03,
            0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0a, 0x0b,
            0x0c, 0x0d, 0x0e, 0x0f,
            0x10, 0x11, 0x12, 0x13,
        ];
        let data = [0x14, 0x15, 0x16, 0x17];
        let event =
            PcrEvent::new_in_buffer(&mut event_buf, PcrIndex(4), EventType::IPL, digest, &data)
                .unwrap();
        assert_eq!(event.pcr_index(), PcrIndex(4));
        assert_eq!(event.event_type(), EventType::IPL);
        assert_eq!(event.digest(), digest);
        assert_eq!(event.event_data(), data);

        let event_ptr: *const PcrEvent = event;
        let bytes =
            unsafe { slice::from_raw_parts(event_ptr.cast::<u8>(), mem::size_of_val(event)) };
        #[rustfmt::skip]
        assert_eq!(bytes, [
            // PCR index
            0x04, 0x00, 0x00, 0x00,
            // Event type
            0x0d, 0x00, 0x00, 0x00,
            // Digest
            0x00, 0x01, 0x02, 0x03,
            0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0a, 0x0b,
            0x0c, 0x0d, 0x0e, 0x0f,
            0x10, 0x11, 0x12, 0x13,
            // Event data len
            0x04, 0x00, 0x00, 0x00,
            // Event data
            0x14, 0x15, 0x16, 0x17,
        ]);
    }

    #[test]
    fn test_event_log_v1() {
        // This data comes from dumping the TPM event log in a VM
        // (truncated to just two entries).
        #[rustfmt::skip]
        let bytes = [
            // Event 1
            // PCR index
            0x00, 0x00, 0x00, 0x00,
            // Event type
            0x08, 0x00, 0x00, 0x00,
            // SHA1 digest
            0x14, 0x89, 0xf9, 0x23, 0xc4, 0xdc, 0xa7, 0x29, 0x17, 0x8b,
            0x3e, 0x32, 0x33, 0x45, 0x85, 0x50, 0xd8, 0xdd, 0xdf, 0x29,
            // Event data size
            0x02, 0x00, 0x00, 0x00,
            // Event data
            0x00, 0x00,

            // Event 2
            // PCR index
            0x00, 0x00, 0x00, 0x00,
            // Event type
            0x08, 0x00, 0x00, 0x80,
            // SHA1 digest
            0xc7, 0x06, 0xe7, 0xdd, 0x36, 0x39, 0x29, 0x84, 0xeb, 0x06,
            0xaa, 0xa0, 0x8f, 0xf3, 0x36, 0x84, 0x40, 0x77, 0xb3, 0xed,
            // Event data size
            0x10, 0x00, 0x00, 0x00,
            // Event data
            0x00, 0x00, 0x82, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x0e, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let log = unsafe { EventLog::new(bytes.as_ptr(), bytes.as_ptr().add(34), false) };
        let mut iter = log.iter();

        // Entry 1
        let entry = iter.next().unwrap();
        assert_eq!(entry.pcr_index(), PcrIndex(0));
        assert_eq!(entry.event_type(), EventType::CRTM_VERSION);
        #[rustfmt::skip]
        assert_eq!(
            entry.digest(),
            [
                0x14, 0x89, 0xf9, 0x23, 0xc4, 0xdc, 0xa7, 0x29, 0x17, 0x8b,
                0x3e, 0x32, 0x33, 0x45, 0x85, 0x50, 0xd8, 0xdd, 0xdf, 0x29,
            ]
        );
        assert_eq!(entry.event_data(), [0x00, 0x00]);

        // Entry 2
        let entry = iter.next().unwrap();
        assert_eq!(entry.pcr_index(), PcrIndex(0));
        assert_eq!(entry.event_type(), EventType::EFI_PLATFORM_FIRMWARE_BLOB);
        #[rustfmt::skip]
        assert_eq!(
            entry.digest(),
            [
                0xc7, 0x06, 0xe7, 0xdd, 0x36, 0x39, 0x29, 0x84, 0xeb, 0x06,
                0xaa, 0xa0, 0x8f, 0xf3, 0x36, 0x84, 0x40, 0x77, 0xb3, 0xed,
            ]
        );
        #[rustfmt::skip]
        assert_eq!(
            entry.event_data(),
            [
                0x00, 0x00, 0x82, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x0e, 0x00, 0x00, 0x00, 0x00, 0x00,
            ]
        );
    }
}
