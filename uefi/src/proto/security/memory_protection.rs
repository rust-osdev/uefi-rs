use crate::data_types::PhysicalAddress;
use crate::proto::unsafe_protocol;
use crate::table::boot::MemoryAttribute;
use crate::{Result, Status};
use core::ops::Range;

/// Protocol for getting and setting memory protection attributes.
///
/// Corresponds to the C type `EFI_MEMORY_ATTRIBUTE_PROTOCOL`.
#[repr(C)]
#[unsafe_protocol("f4560cf6-40ec-4b4a-a192-bf1d57d0b189")]
pub struct MemoryProtection {
    get_memory_attributes: unsafe extern "efiapi" fn(
        this: *const Self,
        base_address: PhysicalAddress,
        length: u64,
        attributes: *mut MemoryAttribute,
    ) -> Status,

    set_memory_attributes: unsafe extern "efiapi" fn(
        this: *const Self,
        base_address: PhysicalAddress,
        length: u64,
        attributes: MemoryAttribute,
    ) -> Status,

    clear_memory_attributes: unsafe extern "efiapi" fn(
        this: *const Self,
        base_address: PhysicalAddress,
        length: u64,
        attributes: MemoryAttribute,
    ) -> Status,
}

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
    /// [UEFI page size]: uefi::table::boot::PAGE_SIZE
    pub fn get_memory_attributes(
        &self,
        byte_region: Range<PhysicalAddress>,
    ) -> Result<MemoryAttribute> {
        let mut attributes = MemoryAttribute::empty();
        let (base_address, length) = range_to_base_and_len(byte_region);
        unsafe {
            (self.get_memory_attributes)(self, base_address, length, &mut attributes)
                .into_with_val(|| attributes)
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
        unsafe { (self.set_memory_attributes)(self, base_address, length, attributes).into() }
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
        unsafe { (self.clear_memory_attributes)(self, base_address, length, attributes).into() }
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
