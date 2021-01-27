impl core::default::Default for EFI_MEMORY_DESCRIPTOR {
    fn default() -> Self {
        Self {
            Type: EFI_MEMORY_TYPE_EfiReservedMemoryType,
            PhysicalStart: 0,
            VirtualStart: 0,
            NumberOfPages: 0,
            Attribute: 0,
        }
    }
}

impl Align for EFI_MEMORY_DESCRIPTOR {
    fn alignment() -> usize {
        core::mem::align_of::<Self>()
    }
}