use crate::data_types::PhysicalAddress;
use crate::proto::unsafe_protocol;
use crate::table::boot::MemoryAttribute;
use crate::{Result, StatusExt};
use core::ops::Range;
use uefi_raw::protocol::memory_protection::MemoryAttributeProtocol;

/// Protocol for getting and setting memory protection attributes.
///
/// Corresponds to the C type `EFI_MEMORY_ATTRIBUTE_PROTOCOL`.
#[repr(transparent)]
#[unsafe_protocol(MemoryAttributeProtocol::GUID)]
pub struct MemoryProtection(MemoryAttributeProtocol);

impl MemoryProtection {
    /// Get the attributes of a memory region.
    ///
    /// The attribute mask this returns will only contain bits in the
    /// set of [`READ_PROTECT`], [`EXECUTE_PROTECT`], and [`READ_ONLY`].
    ///
    /// If the attributes are not consistent within the region,
    /// [`Status::NO_MAPPING`] is returned.
    ///
    /// Implementations typically require that the start and end of the memory
    /// region are aligned to the [UEFI page size].
    ///
    /// [`READ_PROTECT`]: MemoryAttribute::READ_PROTECT
    /// [`EXECUTE_PROTECT`]: MemoryAttribute::EXECUTE_PROTECT
    /// [`READ_ONLY`]: MemoryAttribute::READ_ONLY
    /// [`Status::NO_MAPPING`]: crate::Status::NO_MAPPING
    /// [UEFI page size]: uefi::table::boot::PAGE_SIZE
    pub fn get_memory_attributes(
        &self,
        byte_region: Range<PhysicalAddress>,
    ) -> Result<MemoryAttribute> {
        let mut attributes = MemoryAttribute::empty();
        let (base_address, length) = range_to_base_and_len(byte_region);
        unsafe {
            (self.0.get_memory_attributes)(&self.0, base_address, length, &mut attributes)
                .to_result_with_val(|| attributes)
        }
    }

    /// Set the attributes of a memory region.
    ///
    /// The valid attributes to set are [`READ_PROTECT`],
    /// [`EXECUTE_PROTECT`], and [`READ_ONLY`].
    ///
    /// Implementations typically require that the start and end of the memory
    /// region are aligned to the [UEFI page size].
    ///
    /// [`READ_PROTECT`]: MemoryAttribute::READ_PROTECT
    /// [`EXECUTE_PROTECT`]: MemoryAttribute::EXECUTE_PROTECT
    /// [`READ_ONLY`]: MemoryAttribute::READ_ONLY
    /// [UEFI page size]: uefi::table::boot::PAGE_SIZE
    pub fn set_memory_attributes(
        &self,
        byte_region: Range<PhysicalAddress>,
        attributes: MemoryAttribute,
    ) -> Result {
        let (base_address, length) = range_to_base_and_len(byte_region);
        unsafe {
            (self.0.set_memory_attributes)(&self.0, base_address, length, attributes).to_result()
        }
    }

    /// Clear the attributes of a memory region.
    ///
    /// The valid attributes to clear are [`READ_PROTECT`],
    /// [`EXECUTE_PROTECT`], and [`READ_ONLY`].
    ///
    /// Implementations typically require that the start and end of the memory
    /// region are aligned to the [UEFI page size].
    ///
    /// [`READ_PROTECT`]: MemoryAttribute::READ_PROTECT
    /// [`EXECUTE_PROTECT`]: MemoryAttribute::EXECUTE_PROTECT
    /// [`READ_ONLY`]: MemoryAttribute::READ_ONLY
    /// [UEFI page size]: uefi::table::boot::PAGE_SIZE
    pub fn clear_memory_attributes(
        &self,
        byte_region: Range<PhysicalAddress>,
        attributes: MemoryAttribute,
    ) -> Result {
        let (base_address, length) = range_to_base_and_len(byte_region);
        unsafe {
            (self.0.clear_memory_attributes)(&self.0, base_address, length, attributes).to_result()
        }
    }
}

/// Convert a byte `Range` to `(base_address, length)`.
fn range_to_base_and_len(r: Range<PhysicalAddress>) -> (PhysicalAddress, PhysicalAddress) {
    (r.start, r.end.checked_sub(r.start).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_conversion() {
        assert_eq!(range_to_base_and_len(2..5), (2, 3));
    }
}
