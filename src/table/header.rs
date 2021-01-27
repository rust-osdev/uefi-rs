use super::Revision;
use uefi_sys::EFI_TABLE_HEADER;

/// All standard UEFI tables begin with a common header.
#[derive(Debug)]
#[repr(C)]
pub struct Header {
    /// Unsafe raw type extracted from EDK2
    pub raw: EFI_TABLE_HEADER,
}

impl Header {
    /// Unique identifier for this table.
    pub fn signature(&self) -> u64 {
        self.raw.Signature
    }

    /// Revision of the spec this table conforms to.
    pub fn revision(&self) -> Revision {
        Revision(self.raw.Revision)
    }

    /// The size in bytes of the entire table.
    pub fn size(&self) -> u32 {
        self.raw.HeaderSize
    }

    /// 32-bit CRC-32-Castagnoli of the entire table,
    /// calculated with this field set to 0.
    pub fn crc32(&self) -> u32 {
        self.raw.CRC32
    }
}
