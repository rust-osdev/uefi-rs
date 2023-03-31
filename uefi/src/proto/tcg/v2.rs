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

use super::{v1, AlgorithmId, EventType, HashAlgorithm, PcrIndex};
use crate::data_types::{PhysicalAddress, UnalignedSlice};
use crate::proto::unsafe_protocol;
use crate::util::{ptr_write_unaligned_and_add, usize_from_u32};
use crate::{Error, Result, Status};
use bitflags::bitflags;
use core::fmt::{self, Debug, Formatter};
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::{mem, ptr, slice};
use ptr_meta::{Pointee, PtrExt};

/// Version information.
///
/// Layout compatible with the C type `EFI_TG2_VERSION`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Version {
    /// Major version.
    pub major: u8,
    /// Minor version.
    pub minor: u8,
}

bitflags! {
    /// Event log formats supported by the firmware.
    ///
    /// Corresponds to the C typedef `EFI_TCG2_EVENT_ALGORITHM_BITMAP`.
    #[derive(Default)]
    #[repr(transparent)]
    pub struct EventLogFormat: u32 {
        /// Firmware supports the SHA-1 log format.
        const TCG_1_2 = 0x0000_0001;

        /// Firmware supports the crypto-agile log format.
        const TCG_2 = 0x0000_0002;
    }
}

/// Information about the protocol and the TPM device.
///
/// Layout compatible with the C type `EFI_TCG2_BOOT_SERVICE_CAPABILITY`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BootServiceCapability {
    size: u8,

    /// Version of the EFI TCG2 protocol.
    pub structure_version: Version,

    /// Version of the EFI TCG2 protocol.
    pub protocol_version: Version,

    /// Bitmap of supported hash algorithms.
    pub hash_algorithm_bitmap: HashAlgorithm,

    /// Event log formats supported by the firmware.
    pub supported_event_logs: EventLogFormat,

    present_flag: u8,

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
    pub active_pcr_banks: HashAlgorithm,
}

impl Default for BootServiceCapability {
    fn default() -> Self {
        // OK to unwrap, the size is less than u8.
        let struct_size = u8::try_from(mem::size_of::<BootServiceCapability>()).unwrap();

        Self {
            size: struct_size,
            structure_version: Version::default(),
            protocol_version: Version::default(),
            hash_algorithm_bitmap: HashAlgorithm::default(),
            supported_event_logs: EventLogFormat::default(),
            present_flag: 0,
            max_command_size: 0,
            max_response_size: 0,
            manufacturer_id: 0,
            number_of_pcr_banks: 0,
            active_pcr_banks: HashAlgorithm::default(),
        }
    }
}

impl BootServiceCapability {
    /// Whether the TPM device is present.
    #[must_use]
    pub fn tpm_present(&self) -> bool {
        self.present_flag != 0
    }
}

bitflags! {
    /// Flags for the [`Tcg::hash_log_extend_event`] function.
    #[derive(Default)]
    #[repr(transparent)]
    pub struct HashLogExtendEventFlags: u64 {
        /// Extend an event but don't log it.
        const EFI_TCG2_EXTEND_ONLY = 0x0000_0000_0000_0001;

        /// Use when measuring a PE/COFF image.
        const PE_COFF_IMAGE = 0x0000_0000_0000_0010;
    }
}

/// Header used in [`PcrEventInputs`].
///
/// Layout compatible with the C type `EFI_TCG2_EVENT_HEADER`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C, packed)]
struct EventHeader {
    header_size: u32,
    header_version: u16,
    pcr_index: PcrIndex,
    event_type: EventType,
}

/// Event type passed to [`Tcg::hash_log_extend_event`].
///
/// Layout compatible with the C type `EFI_TCG2_EVENT`.
///
/// The TPM v1 spec uses a single generic event type for both creating a
/// new event and reading an event from the log. The v2 spec splits this
/// into two structs: `EFI_TCG2_EVENT` for creating events, and
/// `TCG_PCR_EVENT2` for reading events. To help clarify the usage, our
/// API renames these types to `PcrEventInputs` and `PcrEvent`,
/// respectively.
#[derive(Pointee)]
#[repr(C, packed)]
pub struct PcrEventInputs {
    size: u32,
    event_header: EventHeader,
    event: [u8],
}

impl PcrEventInputs {
    /// Create a new `PcrEventInputs` using a byte buffer for storage.
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
        event_data: &[u8],
    ) -> Result<&'buf Self> {
        let required_size =
            mem::size_of::<u32>() + mem::size_of::<EventHeader>() + event_data.len();

        if buffer.len() < required_size {
            return Err(Status::BUFFER_TOO_SMALL.into());
        }
        let size_field =
            u32::try_from(required_size).map_err(|_| Error::from(Status::INVALID_PARAMETER))?;

        let mut ptr: *mut u8 = buffer.as_mut_ptr().cast();

        unsafe {
            ptr_write_unaligned_and_add(&mut ptr, size_field);
            ptr_write_unaligned_and_add(
                &mut ptr,
                EventHeader {
                    header_size: u32::try_from(mem::size_of::<EventHeader>()).unwrap(),
                    header_version: 1,
                    pcr_index,
                    event_type,
                },
            );
            ptr::copy(event_data.as_ptr(), ptr, event_data.len());

            let ptr: *const PcrEventInputs =
                ptr_meta::from_raw_parts(buffer.as_ptr().cast(), event_data.len());
            Ok(&*ptr)
        }
    }
}

impl Debug for PcrEventInputs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PcrEventInputs")
            .field("size", &{ self.size })
            .field("event_header", &self.event_header)
            .field("event", &"<binary data>")
            .finish()
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AlgorithmDigestSize {
    algorithm_id: AlgorithmId,
    digest_size: u16,
}

#[derive(Clone, Debug)]
struct AlgorithmDigestSizes<'a>(UnalignedSlice<'a, AlgorithmDigestSize>);

impl<'a> AlgorithmDigestSizes<'a> {
    fn get_size(&self, alg: AlgorithmId) -> Option<u16> {
        self.0.iter().find_map(|elem| {
            if { elem.algorithm_id } == alg {
                Some(elem.digest_size)
            } else {
                None
            }
        })
    }
}

fn u32_le_from_bytes_at_offset(bytes: &[u8], offset: usize) -> Option<u32> {
    let bytes = bytes.get(offset..offset + 4)?;
    // OK to unwrap: we know `bytes` is now of length 4.
    let val = u32::from_le_bytes(bytes.try_into().unwrap());
    Some(val)
}

/// Header stored at the beginning of the event log.
///
/// Layout compatible with the C type `TCG_EfiSpecIDEventStruct`.
#[derive(Clone, Debug)]
#[allow(unused)] // We don't current access most of the fields.
struct EventLogHeader<'a> {
    platform_class: u32,
    // major, minor, errata
    spec_version: (u8, u8, u8),
    uintn_size: u8,
    algorithm_digest_sizes: AlgorithmDigestSizes<'a>,
    vendor_info: &'a [u8],
    // Size of the whole header event, in bytes.
    size_in_bytes: usize,
}

impl<'a> EventLogHeader<'a> {
    fn new(event: &'a v1::PcrEvent) -> Option<Self> {
        if event.pcr_index() != PcrIndex(0) {
            return None;
        }
        if { event.event_type() } != EventType::NO_ACTION {
            return None;
        }
        if event.digest() != [0; 20] {
            return None;
        }

        let event = &event.event_data();
        if event.get(..16)? != *b"Spec ID Event03\0" {
            return None;
        }
        let platform_class = u32_le_from_bytes_at_offset(event, 16)?;
        let version_minor = *event.get(20)?;
        let version_major = *event.get(21)?;
        let version_errata = *event.get(22)?;
        let uintn_size = *event.get(23)?;
        let number_of_algorithms = usize_from_u32(u32_le_from_bytes_at_offset(event, 24)?);
        let vendor_info_size_byte_offset =
            28 + (number_of_algorithms * mem::size_of::<AlgorithmDigestSize>());
        let vendor_info_size = usize::from(*event.get(vendor_info_size_byte_offset)?);

        // Safety: we know the slice is big enough because we just
        // safely got the field after the slice (`vendor_info_size`).
        let algorithm_digest_sizes = unsafe {
            let ptr: *const AlgorithmDigestSize = event.as_ptr().add(28).cast();
            AlgorithmDigestSizes(UnalignedSlice::new(ptr, number_of_algorithms))
        };

        let vendor_info_byte_offset = vendor_info_size_byte_offset + 1;
        let vendor_info =
            event.get(vendor_info_byte_offset..vendor_info_byte_offset + vendor_info_size)?;

        // 32 is the size of PcrEventV1 excluding the event data.
        let size_in_bytes = 32 + vendor_info_byte_offset + vendor_info_size;

        Some(Self {
            platform_class,
            spec_version: (version_major, version_minor, version_errata),
            uintn_size,
            algorithm_digest_sizes,
            vendor_info,
            size_in_bytes,
        })
    }
}

/// TPM event log as returned by [`Tcg::get_event_log_v2`].
///
/// This type of event log can contain multiple hash types (e.g. SHA-1, SHA-256,
/// SHA-512, etc).
#[derive(Debug)]
pub struct EventLog<'a> {
    // Tie the lifetime to the protocol, and by extension, boot services.
    _lifetime: PhantomData<&'a Tcg>,

    location: *const u8,
    last_entry: *const u8,

    is_truncated: bool,
}

impl<'a> EventLog<'a> {
    /// Iterator of events in the log.
    #[must_use]
    pub fn iter(&self) -> EventLogIter {
        if let Some(header) = self.header() {
            // Advance past the header
            let location = unsafe { self.location.add(header.size_in_bytes) };

            EventLogIter {
                log: self,
                location,
                header: self.header(),
            }
        } else {
            EventLogIter {
                log: self,
                location: ptr::null(),
                header: None,
            }
        }
    }

    /// Header at the beginning of the event log.
    fn header(&self) -> Option<EventLogHeader> {
        // The spec is unclear if the header is present when there are
        // no entries, so lets assume that `self.location` will be null
        // if there's no header, and otherwise valid.
        if self.location.is_null() {
            None
        } else {
            // Safety: we trust that the protocol has given us a valid range
            // of memory to read from.
            let event = unsafe { v1::PcrEvent::from_ptr(self.location) };
            EventLogHeader::new(event)
        }
    }

    /// Whether the event log is truncated due to not enough space in the log to
    /// contain some events.
    #[must_use]
    pub fn is_truncated(&self) -> bool {
        self.is_truncated
    }
}

/// Digests in a PCR event.
#[derive(Clone)]
pub struct PcrEventDigests<'a> {
    data: &'a [u8],
    algorithm_digest_sizes: AlgorithmDigestSizes<'a>,
}

impl<'a> Debug for PcrEventDigests<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a> IntoIterator for PcrEventDigests<'a> {
    type Item = (AlgorithmId, &'a [u8]);
    type IntoIter = PcrEventDigestIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PcrEventDigestIter {
            digests: self,
            offset: 0,
        }
    }
}

/// Iterator over a list of digests.
#[derive(Debug)]
pub struct PcrEventDigestIter<'a> {
    digests: PcrEventDigests<'a>,
    offset: usize,
}

impl<'a> Iterator for PcrEventDigestIter<'a> {
    type Item = (AlgorithmId, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        let data = &self.digests.data[self.offset..];
        let alg = data.get(..2)?;
        let alg = AlgorithmId(u16::from_le_bytes([alg[0], alg[1]]));
        let digest_size = usize::from(self.digests.algorithm_digest_sizes.get_size(alg)?);
        let digest = data.get(2..2 + digest_size)?;
        self.offset += 2 + digest_size;
        Some((alg, digest))
    }
}

/// PCR event from an [`EventLog`].
///
/// This roughly corresponds to the C type `TCG_PCR_EVENT2`, but is not layout
/// compatible.
///
/// The TPM v1 spec uses a single generic event type for both creating a
/// new event and reading an event from the log. The v2 spec splits this
/// into two structs: `EFI_TCG2_EVENT` for creating events, and
/// `TCG_PCR_EVENT2` for reading events. To help clarify the usage, our
/// API renames these types to `PcrEventInputs` and `PcrEvent`,
/// respectively.
#[derive(Debug)]
pub struct PcrEvent<'a> {
    pcr_index: PcrIndex,
    event_type: EventType,
    digests: &'a [u8],
    event_data: &'a [u8],

    // Precalculate the pointer to the next event.
    next: *const u8,

    // This data from the v2 log header is needed to parse the digest data.
    algorithm_digest_sizes: AlgorithmDigestSizes<'a>,
}

impl<'a> PcrEvent<'a> {
    unsafe fn from_ptr(ptr: *const u8, header: EventLogHeader<'a>) -> Option<Self> {
        let ptr_u32: *const u32 = ptr.cast();
        let pcr_index = PcrIndex(ptr_u32.read_unaligned());
        let event_type = EventType(ptr_u32.add(1).read_unaligned());
        let digests_count = ptr_u32.add(2).read_unaligned();
        let digests_ptr: *const u8 = ptr.add(12);

        // Get the byte size of the digests so that the digests iterator
        // can be safe code.
        let mut digests_byte_size = 0;
        let mut elem_ptr = digests_ptr;
        for _ in 0..digests_count {
            let algorithm_id = AlgorithmId(elem_ptr.cast::<u16>().read_unaligned());
            let alg_and_digest_size = mem::size_of::<AlgorithmId>()
                + usize::from(header.algorithm_digest_sizes.get_size(algorithm_id)?);
            digests_byte_size += alg_and_digest_size;
            elem_ptr = elem_ptr.add(alg_and_digest_size);
        }

        let digests = slice::from_raw_parts(digests_ptr, digests_byte_size);
        let event_size_ptr = digests_ptr.add(digests_byte_size);
        let event_size = usize_from_u32(event_size_ptr.cast::<u32>().read_unaligned());
        let event_data_ptr = event_size_ptr.add(4);
        let event_data = slice::from_raw_parts(event_data_ptr, event_size);

        Some(Self {
            pcr_index,
            event_type,
            digests,
            event_data,
            next: event_data_ptr.add(event_size),
            algorithm_digest_sizes: header.algorithm_digest_sizes,
        })
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
    /// Note that this data is independent of what is hashed in [`digests`].
    ///
    /// [`digests`]: Self::digests
    /// [`event_type`]: Self::event_type
    #[must_use]
    pub fn event_data(&self) -> &[u8] {
        self.event_data
    }

    /// Digests of the data hashed for this event.
    #[must_use]
    pub fn digests(&self) -> PcrEventDigests {
        PcrEventDigests {
            data: self.digests,
            algorithm_digest_sizes: self.algorithm_digest_sizes.clone(),
        }
    }
}

/// Iterator for events in [`EventLog`].
#[derive(Debug)]
pub struct EventLogIter<'a> {
    log: &'a EventLog<'a>,
    header: Option<EventLogHeader<'a>>,
    location: *const u8,
}

impl<'a> Iterator for EventLogIter<'a> {
    type Item = PcrEvent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // The spec says that `last_entry` will be null if there are no
        // events. Presumably `location` will be null as well, but check
        // both just to be safe.
        if self.location.is_null() || self.log.last_entry.is_null() {
            return None;
        }

        // Safety: we trust that the protocol has given us a valid range
        // of memory to read from.
        let event = unsafe { PcrEvent::from_ptr(self.location, self.header.clone()?)? };

        // If this is the last entry, set the location to null so that
        // future calls to `next()` return `None`.
        if self.location == self.log.last_entry {
            self.location = ptr::null();
        } else {
            self.location = event.next;
        }

        Some(event)
    }
}

/// Protocol for interacting with TPM devices.
///
/// This protocol can be used for interacting with older TPM 1.1/1.2
/// devices, but most firmware only uses it for TPM 2.0.
///
/// The corresponding C type is `EFI_TCG2_PROTOCOL`.
#[repr(C)]
#[unsafe_protocol("607f766c-7455-42be-930b-e4d76db2720f")]
pub struct Tcg {
    get_capability: unsafe extern "efiapi" fn(
        this: *mut Tcg,
        protocol_capability: *mut BootServiceCapability,
    ) -> Status,

    get_event_log: unsafe extern "efiapi" fn(
        this: *mut Tcg,
        event_log_format: EventLogFormat,
        event_log_location: *mut PhysicalAddress,
        event_log_last_entry: *mut PhysicalAddress,
        event_log_truncated: *mut u8,
    ) -> Status,

    hash_log_extend_event: unsafe extern "efiapi" fn(
        this: *mut Tcg,
        flags: HashLogExtendEventFlags,
        data_to_hash: PhysicalAddress,
        data_to_hash_len: u64,
        // Use `()` here rather than `PcrEventInputs` so that it's a
        // thin pointer.
        event: *const (),
    ) -> Status,

    submit_command: unsafe extern "efiapi" fn(
        this: *mut Tcg,
        input_parameter_block_size: u32,
        input_parameter_block: *const u8,
        output_parameter_block_size: u32,
        output_parameter_block: *mut u8,
    ) -> Status,

    get_active_pcr_banks:
        unsafe extern "efiapi" fn(this: *mut Tcg, active_pcr_banks: *mut HashAlgorithm) -> Status,

    set_active_pcr_banks:
        unsafe extern "efiapi" fn(this: *mut Tcg, active_pcr_banks: HashAlgorithm) -> Status,

    get_result_of_set_active_pcr_banks: unsafe extern "efiapi" fn(
        this: *mut Tcg,
        operation_present: *mut u32,
        response: *mut u32,
    ) -> Status,
}

impl Tcg {
    /// Get information about the protocol and TPM device.
    pub fn get_capability(&mut self) -> Result<BootServiceCapability> {
        let mut capability = BootServiceCapability::default();
        unsafe { (self.get_capability)(self, &mut capability).into_with_val(|| capability) }
    }

    /// Get the V1 event log. This provides events in the same format as a V1
    /// TPM, so all events use SHA-1 hashes.
    pub fn get_event_log_v1(&mut self) -> Result<v1::EventLog> {
        let mut location = 0;
        let mut last_entry = 0;
        let mut truncated = 0;

        let status = unsafe {
            (self.get_event_log)(
                self,
                EventLogFormat::TCG_1_2,
                &mut location,
                &mut last_entry,
                &mut truncated,
            )
        };

        if status.is_success() {
            let is_truncated = truncated != 0;

            let log = unsafe {
                v1::EventLog::new(location as *const u8, last_entry as *const u8, is_truncated)
            };

            Ok(log)
        } else {
            Err(status.into())
        }
    }

    /// Get the V2 event log. This format allows for a flexible list of hash types.
    pub fn get_event_log_v2(&mut self) -> Result<EventLog> {
        let mut location = 0;
        let mut last_entry = 0;
        let mut truncated = 0;

        let status = unsafe {
            (self.get_event_log)(
                self,
                EventLogFormat::TCG_2,
                &mut location,
                &mut last_entry,
                &mut truncated,
            )
        };

        if status.is_success() {
            let is_truncated = truncated != 0;

            let log = EventLog {
                _lifetime: PhantomData,
                location: location as *const u8,
                last_entry: last_entry as *const u8,
                is_truncated,
            };

            Ok(log)
        } else {
            Err(status.into())
        }
    }

    /// Extend a PCR and add an entry to the event log.
    pub fn hash_log_extend_event(
        &mut self,
        flags: HashLogExtendEventFlags,
        data_to_hash: &[u8],
        event: &PcrEventInputs,
    ) -> Result {
        let event: *const PcrEventInputs = event;
        let (event, _event_size) = PtrExt::to_raw_parts(event);
        unsafe {
            (self.hash_log_extend_event)(
                self,
                flags,
                data_to_hash.as_ptr() as PhysicalAddress,
                // OK to unwrap, usize fits in u64.
                u64::try_from(data_to_hash.len()).unwrap(),
                event,
            )
            .into()
        }
    }

    /// Send a command directly to the TPM.
    ///
    /// Constructing the input block and parsing the output block are outside
    /// the scope of this crate. See the [TPM 2.0 Specification][spec], in
    /// particular Part 2 (Structures) and Part 3 (Commands).
    ///
    /// Note that TPM structures are big endian.
    ///
    /// [spec]: https://trustedcomputinggroup.org/resource/tpm-library-specification/
    pub fn submit_command(
        &mut self,
        input_parameter_block: &[u8],
        output_parameter_block: &mut [u8],
    ) -> Result {
        let input_parameter_block_len = u32::try_from(input_parameter_block.len())
            .map_err(|_| Error::from(Status::BAD_BUFFER_SIZE))?;
        let output_parameter_block_len = u32::try_from(output_parameter_block.len())
            .map_err(|_| Error::from(Status::BAD_BUFFER_SIZE))?;

        unsafe {
            (self.submit_command)(
                self,
                input_parameter_block_len,
                input_parameter_block.as_ptr(),
                output_parameter_block_len,
                output_parameter_block.as_mut_ptr(),
            )
            .into()
        }
    }

    /// Get a bitmap of the active PCR banks. Each bank corresponds to a hash
    /// algorithm.
    pub fn get_active_pcr_banks(&mut self) -> Result<HashAlgorithm> {
        let mut active_pcr_banks = HashAlgorithm::empty();

        let status = unsafe { (self.get_active_pcr_banks)(self, &mut active_pcr_banks) };

        status.into_with_val(|| active_pcr_banks)
    }

    /// Set the active PCR banks. Each bank corresponds to a hash
    /// algorithm. This change will not take effect until the system is
    /// rebooted twice.
    pub fn set_active_pcr_banks(&mut self, active_pcr_banks: HashAlgorithm) -> Result {
        unsafe { (self.set_active_pcr_banks)(self, active_pcr_banks) }.into()
    }

    /// Get the stored result of calling [`Tcg::set_active_pcr_banks`] in a
    /// previous boot.
    ///
    /// If there was no attempt to set the active PCR banks in a previous boot,
    /// this returns `None`. Otherwise, it returns a numeric response code:
    /// * `0x00000000`: Success
    /// * `0x00000001..=0x00000FFF`: TPM error code
    /// * `0xfffffff0`: The operation was canceled by the user or timed out
    /// * `0xfffffff1`: Firmware error
    pub fn get_result_of_set_active_pcr_banks(&mut self) -> Result<Option<u32>> {
        let mut operation_present = 0;
        let mut response = 0;

        let status = unsafe {
            (self.get_result_of_set_active_pcr_banks)(self, &mut operation_present, &mut response)
        };

        status.into_with_val(|| {
            if operation_present == 0 {
                None
            } else {
                Some(response)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    use core::slice;

    #[test]
    fn test_new_event() {
        let mut buf = [MaybeUninit::uninit(); 22];
        let event_data = [0x12, 0x13, 0x14, 0x15];
        let event =
            PcrEventInputs::new_in_buffer(&mut buf, PcrIndex(4), EventType::IPL, &event_data)
                .unwrap();

        assert_eq!({ event.size }, 22);
        assert_eq!(
            event.event_header,
            EventHeader {
                header_size: 14,
                header_version: 1,
                pcr_index: PcrIndex(4),
                event_type: EventType::IPL,
            }
        );

        // Cast to a byte slice to check the data is exactly as expected.
        let event_ptr: *const PcrEventInputs = event;
        let event_ptr: *const u8 = event_ptr.cast();
        let event_bytes = unsafe { slice::from_raw_parts(event_ptr, mem::size_of_val(event)) };

        #[rustfmt::skip]
        assert_eq!(event_bytes, [
            // Size
            0x16, 0x00, 0x00, 0x00,

            // Header
            // Header size
            0x0e, 0x00, 0x00, 0x00,
            // Header version
            0x01, 0x00,
            // PCR index
            0x04, 0x00, 0x00, 0x00,
            // Event type
            0x0d, 0x00, 0x00, 0x00,
            // Event data
            0x12, 0x13, 0x14, 0x15,
        ]);
    }

    #[test]
    fn test_event_log_v2() {
        // This data comes from dumping the TPM event log in a VM
        // (truncated to just two entries after the header).
        #[rustfmt::skip]
        let bytes = [
            // Header event
            // PCR index
            0x00, 0x00, 0x00, 0x00,
            // Event type
            0x03, 0x00, 0x00, 0x00,
            // SHA1 digest
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Event data size
            0x2d, 0x00, 0x00, 0x00,
            // Spec ID event data
            // Signature
            0x53, 0x70, 0x65, 0x63,
            0x20, 0x49, 0x44, 0x20,
            0x45, 0x76, 0x65, 0x6e,
            0x74, 0x30, 0x33, 0x00,
            // Platform class
            0x00, 0x00, 0x00, 0x00,
            // Spec version (minor, major, errata) (yes the order is weird)
            0x00, 0x02, 0x00,
            // Uintn size
            0x02,
            // Number of algorithms
            0x04, 0x00, 0x00, 0x00,
            // Digest sizes
            // SHA1, size
            0x04, 0x00,
            0x14, 0x00,
            // SHA256, size
            0x0b, 0x00,
            0x20, 0x00,
            // SHA384, size
            0x0c, 0x00,
            0x30, 0x00,
            // SHA512, size
            0x0d, 0x00,
            0x40, 0x00,
            // Vendor info size
            0x00,

            // Event 1
            // PCR index
            0x00, 0x00, 0x00, 0x00,
            // Event type
            0x08, 0x00, 0x00, 0x00,
            // Digest count
            0x04, 0x00, 0x00, 0x00,
            // Digests
            // SHA1
            0x04, 0x00,
            0x14, 0x89, 0xf9, 0x23, 0xc4, 0xdc, 0xa7, 0x29, 0x17, 0x8b,
            0x3e, 0x32, 0x33, 0x45, 0x85, 0x50, 0xd8, 0xdd, 0xdf, 0x29,
            // SHA256
            0x0b, 0x00,
            0x96, 0xa2, 0x96, 0xd2, 0x24, 0xf2, 0x85, 0xc6,
            0x7b, 0xee, 0x93, 0xc3, 0x0f, 0x8a, 0x30, 0x91,
            0x57, 0xf0, 0xda, 0xa3, 0x5d, 0xc5, 0xb8, 0x7e,
            0x41, 0x0b, 0x78, 0x63, 0x0a, 0x09, 0xcf, 0xc7,
            // SHA384
            0x0c, 0x00,
            0x1d, 0xd6, 0xf7, 0xb4, 0x57, 0xad, 0x88, 0x0d,
            0x84, 0x0d, 0x41, 0xc9, 0x61, 0x28, 0x3b, 0xab,
            0x68, 0x8e, 0x94, 0xe4, 0xb5, 0x93, 0x59, 0xea,
            0x45, 0x68, 0x65, 0x81, 0xe9, 0x0f, 0xec, 0xce,
            0xa3, 0xc6, 0x24, 0xb1, 0x22, 0x61, 0x13, 0xf8,
            0x24, 0xf3, 0x15, 0xeb, 0x60, 0xae, 0x0a, 0x7c,
            // SHA512
            0x0d, 0x00,
            0x5e, 0xa7, 0x1d, 0xc6, 0xd0, 0xb4, 0xf5, 0x7b,
            0xf3, 0x9a, 0xad, 0xd0, 0x7c, 0x20, 0x8c, 0x35,
            0xf0, 0x6c, 0xd2, 0xba, 0xc5, 0xfd, 0xe2, 0x10,
            0x39, 0x7f, 0x70, 0xde, 0x11, 0xd4, 0x39, 0xc6,
            0x2e, 0xc1, 0xcd, 0xf3, 0x18, 0x37, 0x58, 0x86,
            0x5f, 0xd3, 0x87, 0xfc, 0xea, 0x0b, 0xad, 0xa2,
            0xf6, 0xc3, 0x7a, 0x4a, 0x17, 0x85, 0x1d, 0xd1,
            0xd7, 0x8f, 0xef, 0xe6, 0xf2, 0x04, 0xee, 0x54,
            // Event size
            0x02, 0x00, 0x00, 0x00,
            // Event data
            0x00, 0x00,

            // Event 2
            // PCR index
            0x00, 0x00, 0x00, 0x00,
            // Event type
            0x08, 0x00, 0x00, 0x80,
            // Digest count
            0x04, 0x00, 0x00, 0x00,
            // SHA1
            0x04, 0x00,
            0xc7, 0x06, 0xe7, 0xdd, 0x36, 0x39, 0x29, 0x84, 0xeb, 0x06,
            0xaa, 0xa0, 0x8f, 0xf3, 0x36, 0x84, 0x40, 0x77, 0xb3, 0xed,
            // SHA256
            0x0b, 0x00,
            0x3a, 0x30, 0x8e, 0x95, 0x87, 0x84, 0xbf, 0xd0,
            0xf6, 0xe3, 0xf1, 0xbd, 0x4d, 0x42, 0x14, 0xd3,
            0x0a, 0x4c, 0x55, 0x00, 0xa4, 0x5b, 0x06, 0xda,
            0x96, 0xfc, 0x90, 0x33, 0x8f, 0x69, 0xb3, 0x61,
            // SHA384
            0x0c, 0x00,
            0xc0, 0xd0, 0x75, 0x96, 0xc5, 0x9a, 0x90, 0x7b,
            0x79, 0x71, 0x6f, 0xc9, 0xf3, 0x6a, 0xad, 0x8f,
            0x0f, 0x26, 0xf2, 0x02, 0x67, 0x5b, 0x42, 0x5a,
            0x52, 0x3f, 0x72, 0xec, 0xb6, 0xf2, 0x53, 0x99,
            0x57, 0xf0, 0xd9, 0x2c, 0x0a, 0x7d, 0xce, 0xaa,
            0xf9, 0x9e, 0x60, 0x0e, 0x54, 0x18, 0xf1, 0xdc,
            // SHA512
            0x0d, 0x00,
            0x9a, 0xe9, 0x25, 0xdc, 0x9c, 0xd2, 0x9d, 0xf0,
            0xe5, 0x80, 0x54, 0x35, 0xa5, 0x99, 0x06, 0x1f,
            0xcf, 0x32, 0x98, 0xcc, 0x2a, 0x15, 0xe4, 0x87,
            0x99, 0xa2, 0x0c, 0x9c, 0xe5, 0x6c, 0x8f, 0xe5,
            0x84, 0x09, 0x75, 0xaf, 0xf0, 0xe1, 0xb6, 0x98,
            0x20, 0x07, 0x5e, 0xe4, 0x29, 0x79, 0x8b, 0x5d,
            0xbb, 0xe5, 0xd1, 0xa2, 0x74, 0x36, 0xab, 0x49,
            0xf1, 0x9b, 0x7a, 0x04, 0x11, 0xd2, 0x96, 0x2c,
            // Event size
            0x10, 0x00, 0x00, 0x00,
            // Event data
            0x00, 0x00, 0x82, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x0e, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let log = EventLog {
            _lifetime: PhantomData,
            location: bytes.as_ptr(),
            last_entry: unsafe { bytes.as_ptr().add(267) },
            is_truncated: false,
        };

        let header = log.header().unwrap();
        assert_eq!(header.platform_class, 0);
        assert_eq!(header.spec_version, (2, 0, 0));
        assert_eq!(header.uintn_size, 2);
        assert_eq!(
            header.algorithm_digest_sizes.0.to_vec(),
            [
                AlgorithmDigestSize {
                    algorithm_id: AlgorithmId::SHA1,
                    digest_size: 20,
                },
                AlgorithmDigestSize {
                    algorithm_id: AlgorithmId::SHA256,
                    digest_size: 32,
                },
                AlgorithmDigestSize {
                    algorithm_id: AlgorithmId::SHA384,
                    digest_size: 48,
                },
                AlgorithmDigestSize {
                    algorithm_id: AlgorithmId::SHA512,
                    digest_size: 64,
                },
            ]
        );
        assert_eq!(header.vendor_info, []);

        let mut iter = log.iter();

        // Entry 1
        let entry = iter.next().unwrap();
        assert_eq!(entry.pcr_index, PcrIndex(0));
        assert_eq!(entry.event_type, EventType::CRTM_VERSION);
        #[rustfmt::skip]
        assert_eq!(
            entry.digests().into_iter().collect::<Vec<_>>(),
            [
                (AlgorithmId::SHA1, [
                    0x14, 0x89, 0xf9, 0x23, 0xc4, 0xdc, 0xa7, 0x29, 0x17, 0x8b,
                    0x3e, 0x32, 0x33, 0x45, 0x85, 0x50, 0xd8, 0xdd, 0xdf, 0x29,
                ].as_slice()),
                (AlgorithmId::SHA256, [
                    0x96, 0xa2, 0x96, 0xd2, 0x24, 0xf2, 0x85, 0xc6,
                    0x7b, 0xee, 0x93, 0xc3, 0x0f, 0x8a, 0x30, 0x91,
                    0x57, 0xf0, 0xda, 0xa3, 0x5d, 0xc5, 0xb8, 0x7e,
                    0x41, 0x0b, 0x78, 0x63, 0x0a, 0x09, 0xcf, 0xc7,
                ].as_slice()),
                (AlgorithmId::SHA384, [
                    0x1d, 0xd6, 0xf7, 0xb4, 0x57, 0xad, 0x88, 0x0d,
                    0x84, 0x0d, 0x41, 0xc9, 0x61, 0x28, 0x3b, 0xab,
                    0x68, 0x8e, 0x94, 0xe4, 0xb5, 0x93, 0x59, 0xea,
                    0x45, 0x68, 0x65, 0x81, 0xe9, 0x0f, 0xec, 0xce,
                    0xa3, 0xc6, 0x24, 0xb1, 0x22, 0x61, 0x13, 0xf8,
                    0x24, 0xf3, 0x15, 0xeb, 0x60, 0xae, 0x0a, 0x7c,
                ].as_slice()),
                (AlgorithmId::SHA512, [
                    0x5e, 0xa7, 0x1d, 0xc6, 0xd0, 0xb4, 0xf5, 0x7b,
                    0xf3, 0x9a, 0xad, 0xd0, 0x7c, 0x20, 0x8c, 0x35,
                    0xf0, 0x6c, 0xd2, 0xba, 0xc5, 0xfd, 0xe2, 0x10,
                    0x39, 0x7f, 0x70, 0xde, 0x11, 0xd4, 0x39, 0xc6,
                    0x2e, 0xc1, 0xcd, 0xf3, 0x18, 0x37, 0x58, 0x86,
                    0x5f, 0xd3, 0x87, 0xfc, 0xea, 0x0b, 0xad, 0xa2,
                    0xf6, 0xc3, 0x7a, 0x4a, 0x17, 0x85, 0x1d, 0xd1,
                    0xd7, 0x8f, 0xef, 0xe6, 0xf2, 0x04, 0xee, 0x54,
                ].as_slice()),
            ]
        );
        assert_eq!(entry.event_data, [0, 0]);

        // Entry 2
        let entry = iter.next().unwrap();
        assert_eq!(entry.pcr_index, PcrIndex(0));
        assert_eq!(entry.event_type, EventType::EFI_PLATFORM_FIRMWARE_BLOB);
        #[rustfmt::skip]
        assert_eq!(
            entry.digests().into_iter().collect::<Vec<_>>(),
            [
                (AlgorithmId::SHA1, [
                    0xc7, 0x06, 0xe7, 0xdd, 0x36, 0x39, 0x29, 0x84, 0xeb, 0x06,
                    0xaa, 0xa0, 0x8f, 0xf3, 0x36, 0x84, 0x40, 0x77, 0xb3, 0xed,
                ].as_slice()),
                (AlgorithmId::SHA256, [
                    0x3a, 0x30, 0x8e, 0x95, 0x87, 0x84, 0xbf, 0xd0,
                    0xf6, 0xe3, 0xf1, 0xbd, 0x4d, 0x42, 0x14, 0xd3,
                    0x0a, 0x4c, 0x55, 0x00, 0xa4, 0x5b, 0x06, 0xda,
                    0x96, 0xfc, 0x90, 0x33, 0x8f, 0x69, 0xb3, 0x61,
                ].as_slice()),
                (AlgorithmId::SHA384, [
                    0xc0, 0xd0, 0x75, 0x96, 0xc5, 0x9a, 0x90, 0x7b,
                    0x79, 0x71, 0x6f, 0xc9, 0xf3, 0x6a, 0xad, 0x8f,
                    0x0f, 0x26, 0xf2, 0x02, 0x67, 0x5b, 0x42, 0x5a,
                    0x52, 0x3f, 0x72, 0xec, 0xb6, 0xf2, 0x53, 0x99,
                    0x57, 0xf0, 0xd9, 0x2c, 0x0a, 0x7d, 0xce, 0xaa,
                    0xf9, 0x9e, 0x60, 0x0e, 0x54, 0x18, 0xf1, 0xdc,
                ].as_slice()),
                (AlgorithmId::SHA512, [
                    0x9a, 0xe9, 0x25, 0xdc, 0x9c, 0xd2, 0x9d, 0xf0,
                    0xe5, 0x80, 0x54, 0x35, 0xa5, 0x99, 0x06, 0x1f,
                    0xcf, 0x32, 0x98, 0xcc, 0x2a, 0x15, 0xe4, 0x87,
                    0x99, 0xa2, 0x0c, 0x9c, 0xe5, 0x6c, 0x8f, 0xe5,
                    0x84, 0x09, 0x75, 0xaf, 0xf0, 0xe1, 0xb6, 0x98,
                    0x20, 0x07, 0x5e, 0xe4, 0x29, 0x79, 0x8b, 0x5d,
                    0xbb, 0xe5, 0xd1, 0xa2, 0x74, 0x36, 0xab, 0x49,
                    0xf1, 0x9b, 0x7a, 0x04, 0x11, 0xd2, 0x96, 0x2c,
                ].as_slice()),
            ]
        );
        #[rustfmt::skip]
        assert_eq!(entry.event_data, [
            0x00, 0x00, 0x82, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x0e, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]);

        assert!(iter.next().is_none());
    }
}
