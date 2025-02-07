// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bundles all relevant types and helpers to work with the UEFI memory map.
//!
//! To work with the memory map, you should use one of the structs
//! [`MemoryMapOwned`], [`MemoryMapRef`], or [`MemoryMapRefMut`] - depending on
//! your use-case. The  traits [`MemoryMap`] and [`MemoryMapMut`] mainly exist
//! to guarantee a streamlined API across these types. We recommend to work with
//! the specific implementation.
//!
//! # Usecase: Obtain UEFI Memory Map
//!
//! You can use [`boot::exit_boot_services`] or
//! [`boot::memory_map`], which returns an properly initialized
//! [`MemoryMapOwned`].
//!
//! # Usecase: Parse Memory Slice as UEFI Memory Map
//!
//! If you have a chunk of memory and want to parse it as UEFI memory map, which
//! might be the case if a bootloader such as GRUB or Limine passes its boot
//! information, you can use [`MemoryMapRef`] or [`MemoryMapRefMut`].
//!
//! # All relevant exports:
//!
//! - the traits [`MemoryMap`] and [`MemoryMapMut`],
//! - the trait implementations [`MemoryMapOwned`], [`MemoryMapRef`], and
//!   [`MemoryMapRefMut`],
//! - the iterator [`MemoryMapIter`]
//! - various associated helper types, such as [`MemoryMapKey`] and
//!   [`MemoryMapMeta`],
//! - re-exports [`MemoryDescriptor`], [`MemoryType`], and [`MemoryAttribute`].
//!
//! [`boot::exit_boot_services`]: crate::boot::exit_boot_services
//! [`boot::memory_map`]: crate::boot::memory_map

mod api;
mod impl_;
mod iter;

pub use api::*;
pub use impl_::*;
pub use iter::*;
pub use uefi_raw::table::boot::{MemoryAttribute, MemoryDescriptor, MemoryType};

use crate::data_types::Align;

impl Align for MemoryDescriptor {
    fn alignment() -> usize {
        align_of::<Self>()
    }
}

/// A unique identifier of a UEFI memory map, used to tell the firmware that one
/// has the latest valid memory map when exiting boot services.
///
/// If the memory map changes, due to any allocation or deallocation, this value
/// is no longer valid, and exiting boot services will fail.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct MemoryMapKey(pub(crate) usize);

/// A structure containing the meta attributes associated with a call to
/// `GetMemoryMap` of UEFI. Note that all values refer to the time this was
/// called. All following invocations (hidden, subtle, and asynchronous ones)
/// will likely invalidate this.
#[derive(Copy, Clone, Debug)]
pub struct MemoryMapMeta {
    /// The actual size of the map.
    pub map_size: usize,
    /// The reported memory descriptor size. Note that this is the reference
    /// and never `size_of::<MemoryDescriptor>()`!
    pub desc_size: usize,
    /// A unique memory key bound to a specific memory map version/state.
    pub map_key: MemoryMapKey,
    /// The version of the descriptor struct.
    pub desc_version: u32,
}

impl MemoryMapMeta {
    /// Returns the amount of entries in the map.
    #[must_use]
    pub fn entry_count(&self) -> usize {
        assert_eq!(self.map_size % self.desc_size, 0);
        self.map_size / self.desc_size
    }

    /// Runs some sanity assertions.
    pub fn assert_sanity_checks(&self) {
        assert!(self.desc_size > 0);
        // Although very unlikely, this might fail if the memory descriptor is
        // extended by a future UEFI revision by a significant amount, we
        // update the struct, but an old UEFI implementation reports a small
        // size.
        assert!(self.desc_size >= size_of::<MemoryDescriptor>());
        assert!(self.map_size > 0);

        // Ensure the mmap size is (somehow) sane.
        const ONE_GB: usize = 1024 * 1024 * 1024;
        assert!(self.map_size <= ONE_GB);
    }
}

/// Comprehensive unit test of the memory map functionality with the simplified
/// data. Here, `desc_size` equals `size_of::<MemoryDescriptor`.
#[cfg(test)]
mod tests_mmap_artificial {
    use super::*;
    use core::mem::{size_of, size_of_val};

    fn buffer_to_map(buffer: &mut [MemoryDescriptor]) -> MemoryMapRefMut {
        let mmap_len = size_of_val(buffer);
        let mmap = {
            unsafe { core::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, mmap_len) }
        };

        MemoryMapRefMut::new(
            mmap,
            MemoryMapMeta {
                map_size: mmap_len,
                desc_size: size_of::<MemoryDescriptor>(),
                map_key: Default::default(),
                desc_version: MemoryDescriptor::VERSION,
            },
        )
        .unwrap()
    }

    #[test]
    fn mem_map_sorting() {
        // Doesn't matter what type it is.
        const TY: MemoryType = MemoryType::RESERVED;

        const BASE: MemoryDescriptor = MemoryDescriptor {
            ty: TY,
            phys_start: 0,
            virt_start: 0,
            page_count: 0,
            att: MemoryAttribute::empty(),
        };

        let mut buffer = [
            MemoryDescriptor {
                phys_start: 2000,
                ..BASE
            },
            MemoryDescriptor {
                phys_start: 3000,
                ..BASE
            },
            BASE,
            MemoryDescriptor {
                phys_start: 1000,
                ..BASE
            },
        ];

        let mut mem_map = buffer_to_map(&mut buffer);

        mem_map.sort();

        if !is_sorted(&mem_map.entries()) {
            panic!("mem_map is not sorted: {:?}", mem_map);
        }
    }

    #[test]
    fn mem_map_get() {
        // Doesn't matter what type it is.
        const TY: MemoryType = MemoryType::RESERVED;

        const BASE: MemoryDescriptor = MemoryDescriptor {
            ty: TY,
            phys_start: 0,
            virt_start: 0,
            page_count: 0,
            att: MemoryAttribute::empty(),
        };

        const BUFFER: [MemoryDescriptor; 4] = [
            MemoryDescriptor {
                phys_start: 2000,
                ..BASE
            },
            MemoryDescriptor {
                phys_start: 3000,
                ..BASE
            },
            BASE,
            MemoryDescriptor {
                phys_start: 1000,
                ..BASE
            },
        ];

        let mut buffer = BUFFER;

        let mut mem_map = buffer_to_map(&mut buffer);

        for index in 0..3 {
            assert_eq!(mem_map.get(index), BUFFER.get(index));

            // Test Index impl
            assert_eq!(Some(&mem_map[index]), BUFFER.get(index));
        }

        let mut_desc = mem_map.get_mut(2).unwrap();

        mut_desc.phys_start = 300;

        let desc = mem_map.get(2).unwrap();

        assert_ne!(*desc, BUFFER[2]);
    }

    fn is_sorted(iter: &MemoryMapIter) -> bool {
        let mut iter = iter.clone();
        let mut curr_start;

        if let Some(val) = iter.next() {
            curr_start = val.phys_start;
        } else {
            return true;
        }

        for desc in iter {
            if desc.phys_start <= curr_start {
                return false;
            }
            curr_start = desc.phys_start
        }
        true
    }
}

/// Comprehensive unit test of the memory map functionality with the data from a
/// real UEFI memory map. The important property that we test here is that
/// the reported `desc_size` doesn't equal `size_of::<MemoryDescriptor`.
#[cfg(test)]
mod tests_mmap_real {
    use super::*;
    use alloc::vec::Vec;
    use core::slice;
    use size_of;

    const MMAP_META: MemoryMapMeta = MemoryMapMeta {
        map_size: MMAP_RAW.len() * size_of::<u64>(),
        desc_size: 48,
        map_key: MemoryMapKey(0),
        desc_version: 1,
    };
    /// Sample with 10 entries of a real UEFI memory map extracted from our
    /// UEFI test runner.
    const MMAP_RAW: [u64; 60] = [
        3, 0, 0, 1, 15, 0, 7, 4096, 0, 134, 15, 0, 4, 552960, 0, 1, 15, 0, 7, 557056, 0, 24, 15, 0,
        7, 1048576, 0, 1792, 15, 0, 10, 8388608, 0, 8, 15, 0, 7, 8421376, 0, 3, 15, 0, 10, 8433664,
        0, 1, 15, 0, 7, 8437760, 0, 4, 15, 0, 10, 8454144, 0, 240, 15, 0,
    ];

    #[test]
    fn basic_functionality() {
        let mut buf = MMAP_RAW;
        let buf =
            unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr().cast::<u8>(), MMAP_META.map_size) };
        let mut mmap = MemoryMapRefMut::new(buf, MMAP_META).unwrap();

        assert!(mmap.is_sorted());
        mmap.sort();
        assert!(mmap.is_sorted());

        let entries = mmap.entries().copied().collect::<Vec<_>>();

        let expected = [
            MemoryDescriptor {
                ty: MemoryType::BOOT_SERVICES_CODE,
                phys_start: 0x0,
                virt_start: 0x0,
                page_count: 0x1,
                att: MemoryAttribute::UNCACHEABLE
                    | MemoryAttribute::WRITE_COMBINE
                    | MemoryAttribute::WRITE_THROUGH
                    | MemoryAttribute::WRITE_BACK,
            },
            MemoryDescriptor {
                ty: MemoryType::CONVENTIONAL,
                phys_start: 0x1000,
                virt_start: 0x0,
                page_count: 0x86,
                att: MemoryAttribute::UNCACHEABLE
                    | MemoryAttribute::WRITE_COMBINE
                    | MemoryAttribute::WRITE_THROUGH
                    | MemoryAttribute::WRITE_BACK,
            },
            MemoryDescriptor {
                ty: MemoryType::BOOT_SERVICES_DATA,
                phys_start: 0x87000,
                virt_start: 0x0,
                page_count: 0x1,
                att: MemoryAttribute::UNCACHEABLE
                    | MemoryAttribute::WRITE_COMBINE
                    | MemoryAttribute::WRITE_THROUGH
                    | MemoryAttribute::WRITE_BACK,
            },
            MemoryDescriptor {
                ty: MemoryType::CONVENTIONAL,
                phys_start: 0x88000,
                virt_start: 0x0,
                page_count: 0x18,
                att: MemoryAttribute::UNCACHEABLE
                    | MemoryAttribute::WRITE_COMBINE
                    | MemoryAttribute::WRITE_THROUGH
                    | MemoryAttribute::WRITE_BACK,
            },
            MemoryDescriptor {
                ty: MemoryType::CONVENTIONAL,
                phys_start: 0x100000,
                virt_start: 0x0,
                page_count: 0x700,
                att: MemoryAttribute::UNCACHEABLE
                    | MemoryAttribute::WRITE_COMBINE
                    | MemoryAttribute::WRITE_THROUGH
                    | MemoryAttribute::WRITE_BACK,
            },
            MemoryDescriptor {
                ty: MemoryType::ACPI_NON_VOLATILE,
                phys_start: 0x800000,
                virt_start: 0x0,
                page_count: 0x8,
                att: MemoryAttribute::UNCACHEABLE
                    | MemoryAttribute::WRITE_COMBINE
                    | MemoryAttribute::WRITE_THROUGH
                    | MemoryAttribute::WRITE_BACK,
            },
            MemoryDescriptor {
                ty: MemoryType::CONVENTIONAL,
                phys_start: 0x808000,
                virt_start: 0x0,
                page_count: 0x3,
                att: MemoryAttribute::UNCACHEABLE
                    | MemoryAttribute::WRITE_COMBINE
                    | MemoryAttribute::WRITE_THROUGH
                    | MemoryAttribute::WRITE_BACK,
            },
            MemoryDescriptor {
                ty: MemoryType::ACPI_NON_VOLATILE,
                phys_start: 0x80b000,
                virt_start: 0x0,
                page_count: 0x1,
                att: MemoryAttribute::UNCACHEABLE
                    | MemoryAttribute::WRITE_COMBINE
                    | MemoryAttribute::WRITE_THROUGH
                    | MemoryAttribute::WRITE_BACK,
            },
            MemoryDescriptor {
                ty: MemoryType::CONVENTIONAL,
                phys_start: 0x80c000,
                virt_start: 0x0,
                page_count: 0x4,
                att: MemoryAttribute::UNCACHEABLE
                    | MemoryAttribute::WRITE_COMBINE
                    | MemoryAttribute::WRITE_THROUGH
                    | MemoryAttribute::WRITE_BACK,
            },
            MemoryDescriptor {
                ty: MemoryType::ACPI_NON_VOLATILE,
                phys_start: 0x810000,
                virt_start: 0x0,
                page_count: 0xf0,
                att: MemoryAttribute::UNCACHEABLE
                    | MemoryAttribute::WRITE_COMBINE
                    | MemoryAttribute::WRITE_THROUGH
                    | MemoryAttribute::WRITE_BACK,
            },
        ];
        assert_eq!(entries.as_slice(), &expected);
    }
}
