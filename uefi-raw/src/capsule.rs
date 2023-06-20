//! UEFI update capsules.
//!
//! Capsules are used to pass information to the firmware, for example to
//! trigger a firmware update.

use crate::{Guid, PhysicalAddress};
use bitflags::bitflags;

/// Descriptor that defines a scatter-gather list for passing a set of capsules
/// to the firmware.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct CapsuleBlockDescriptor {
    /// Size in bytes of the data block. If zero, the block is treated as a
    /// continuation pointer.
    pub length: u64,

    /// Either a data block pointer or a continuation pointer.
    ///
    /// * If `length` is non-zero, this is the physical address of the data
    /// block.
    /// * If `length` is zero:
    ///   * If `addr` is non-zero, this is the physical address of another block
    ///     of `CapsuleBlockDescriptor`.
    ///   * If `addr` is zero, this entry represents the end of the list.
    pub address: PhysicalAddress,
}

bitflags! {
    /// Capsule update flags.
    ///
    /// The meaning of bits `0..=15` are defined by the capsule GUID.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(transparent)]
    pub struct CapsuleFlags: u32 {
        /// The meaning of this bit depends on the capsule GUID.
        const TYPE_SPECIFIC_BIT_0 = 1 << 0;

        /// The meaning of this bit depends on the capsule GUID.
        const TYPE_SPECIFIC_BIT_1 = 1 << 1;

        /// The meaning of this bit depends on the capsule GUID.
        const TYPE_SPECIFIC_BIT_2 = 1 << 2;

        /// The meaning of this bit depends on the capsule GUID.
        const TYPE_SPECIFIC_BIT_3 = 1 << 3;

        /// The meaning of this bit depends on the capsule GUID.
        const TYPE_SPECIFIC_BIT_4 = 1 << 4;

        /// The meaning of this bit depends on the capsule GUID.
        const TYPE_SPECIFIC_BIT_5 = 1 << 5;

        /// The meaning of this bit depends on the capsule GUID.
        const TYPE_SPECIFIC_BIT_6 = 1 << 6;

        /// The meaning of this bit depends on the capsule GUID.
        const TYPE_SPECIFIC_BIT_7 = 1 << 7;

        /// The meaning of this bit depends on the capsule GUID.
        const TYPE_SPECIFIC_BIT_8 = 1 << 8;

        /// The meaning of this bit depends on the capsule GUID.
        const TYPE_SPECIFIC_BIT_9 = 1 << 9;

        /// The meaning of this bit depends on the capsule GUID.
        const TYPE_SPECIFIC_BIT_10 = 1 << 10;

        /// The meaning of this bit depends on the capsule GUID.
        const TYPE_SPECIFIC_BIT_11 = 1 << 11;

        /// The meaning of this bit depends on the capsule GUID.
        const TYPE_SPECIFIC_BIT_12 = 1 << 12;

        /// The meaning of this bit depends on the capsule GUID.
        const TYPE_SPECIFIC_BIT_13 = 1 << 13;

        /// The meaning of this bit depends on the capsule GUID.
        const TYPE_SPECIFIC_BIT_14 = 1 << 14;

        /// The meaning of this bit depends on the capsule GUID.
        const TYPE_SPECIFIC_BIT_15 = 1 << 15;

        /// Indicates the firmware should process the capsule after system reset.
        const PERSIST_ACROSS_RESET = 1 << 16;

        /// Causes the contents of the capsule to be coalesced from the
        /// scatter-gather list into a contiguous buffer, and then a pointer to
        /// that buffer will be placed in the configuration table after system
        /// reset.
        ///
        /// If this flag is set, [`PERSIST_ACROSS_RESET`] must be set as well.
        ///
        /// [`PERSIST_ACROSS_RESET`]: Self::PERSIST_ACROSS_RESET
        const POPULATE_SYSTEM_TABLE = 1 << 17;

        /// Trigger a system reset after passing the capsule to the firmware.
        ///
        /// If this flag is set, [`PERSIST_ACROSS_RESET`] must be set as well.
        ///
        /// [`PERSIST_ACROSS_RESET`]: Self::PERSIST_ACROSS_RESET
        const INITIATE_RESET = 1 << 18;
    }
}

/// Common header at the start of a capsule.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct CapsuleHeader {
    /// GUID that defines the type of data in the capsule.
    pub capsule_guid: Guid,

    /// Size in bytes of the capsule header. This may be larger than the size of
    /// `CapsuleHeader` since the specific capsule type defined by
    /// [`capsule_guid`] may add additional header fields.
    ///
    /// [`capsule_guid`]: Self::capsule_guid
    pub header_size: u32,

    /// Capsule update flags.
    pub flags: CapsuleFlags,

    /// Size in bytes of the entire capsule, including the header.
    pub capsule_image_size: u32,
}
