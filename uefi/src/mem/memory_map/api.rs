// SPDX-License-Identifier: MIT OR Apache-2.0

//! Module for the traits [`MemoryMap`] and [`MemoryMapMut`].

use super::*;
use core::fmt::Debug;
use core::ops::{Index, IndexMut};

/// An accessory to the UEFI memory map and associated metadata that can be
/// either iterated or indexed like an array.
///
/// A [`MemoryMap`] is always associated with the unique [`MemoryMapKey`]
/// bundled with the map.
///
/// To iterate over the entries, call [`MemoryMap::entries`].
///
/// ## UEFI pitfalls
/// Note that a MemoryMap can quickly become outdated, as soon as any explicit
/// or hidden allocation happens.
///
/// As soon as boot services are excited, all previous obtained memory maps must
/// be considered as outdated, except if the [`MemoryMapKey`] equals the one
/// returned by `exit_boot_services()`.
///
/// **Please note** that when working with memory maps, the `entry_size` is
/// usually larger than `size_of::<MemoryDescriptor` [[0]]. So to be safe,
/// always use `entry_size` as step-size when interfacing with the memory map on
/// a low level.
///
/// [0]: https://github.com/tianocore/edk2/blob/7142e648416ff5d3eac6c6d607874805f5de0ca8/MdeModulePkg/Core/PiSmmCore/Page.c#L1059
pub trait MemoryMap: Debug + Index<usize, Output = MemoryDescriptor> {
    /// Returns the associated [`MemoryMapMeta`].
    #[must_use]
    fn meta(&self) -> MemoryMapMeta;

    /// Returns the associated [`MemoryMapKey`]. Note that this isn't
    /// necessarily the key of the latest valid UEFI memory map.
    #[must_use]
    fn key(&self) -> MemoryMapKey;

    /// Returns the number of keys in the map.
    #[must_use]
    fn len(&self) -> usize;

    /// Returns if the memory map is empty.
    #[must_use]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the [`MemoryDescriptor`] at the given index, if
    /// present.
    #[must_use]
    fn get(&self, index: usize) -> Option<&MemoryDescriptor> {
        if index >= self.len() {
            None
        } else {
            let offset = index * self.meta().desc_size;
            unsafe {
                self.buffer()
                    .as_ptr()
                    .add(offset)
                    .cast::<MemoryDescriptor>()
                    .as_ref()
            }
        }
    }

    /// Returns a reference to the underlying memory.
    #[must_use]
    fn buffer(&self) -> &[u8];

    /// Returns an Iterator of type [`MemoryMapIter`].
    #[must_use]
    fn entries(&self) -> MemoryMapIter<'_>;

    /// Returns if the underlying memory map is sorted regarding the physical
    /// address start.
    #[must_use]
    fn is_sorted(&self) -> bool {
        let iter = self.entries();
        let iter = iter.clone().zip(iter.skip(1));

        for (curr, next) in iter {
            if next.phys_start < curr.phys_start {
                log::debug!("next.phys_start < curr.phys_start: curr={curr:?}, next={next:?}");
                return false;
            }
        }
        true
    }
}

/// Extension to [`MemoryMap`] that adds mutable operations. This also includes
/// the ability to sort the memory map.
pub trait MemoryMapMut: MemoryMap + IndexMut<usize> {
    /// Returns a mutable reference to the [`MemoryDescriptor`] at the given
    /// index, if present.
    #[must_use]
    fn get_mut(&mut self, index: usize) -> Option<&mut MemoryDescriptor> {
        if index >= self.len() {
            None
        } else {
            let offset = index * self.meta().desc_size;
            unsafe {
                self.buffer_mut()
                    .as_mut_ptr()
                    .add(offset)
                    .cast::<MemoryDescriptor>()
                    .as_mut()
            }
        }
    }

    /// Sorts the memory map by physical address in place. This operation is
    /// optional and should be invoked only once.
    fn sort(&mut self);

    /// Returns a reference to the underlying memory.
    ///
    /// # Safety
    ///
    /// This is unsafe as there is a potential to create invalid entries.
    unsafe fn buffer_mut(&mut self) -> &mut [u8];
}
