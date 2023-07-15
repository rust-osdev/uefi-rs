use crate::table::boot::MemoryAttribute;
use crate::{guid, Guid, PhysicalAddress, Status};

#[repr(C)]
pub struct MemoryAttributeProtocol {
    pub get_memory_attributes: unsafe extern "efiapi" fn(
        this: *const Self,
        base_address: PhysicalAddress,
        length: u64,
        attributes: *mut MemoryAttribute,
    ) -> Status,

    pub set_memory_attributes: unsafe extern "efiapi" fn(
        this: *const Self,
        base_address: PhysicalAddress,
        length: u64,
        attributes: MemoryAttribute,
    ) -> Status,

    pub clear_memory_attributes: unsafe extern "efiapi" fn(
        this: *const Self,
        base_address: PhysicalAddress,
        length: u64,
        attributes: MemoryAttribute,
    ) -> Status,
}

impl MemoryAttributeProtocol {
    pub const GUID: Guid = guid!("f4560cf6-40ec-4b4a-a192-bf1d57d0b189");
}
