// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

/// An iterator for [`MemoryMap`].
///
/// The underlying memory might contain an invalid/malformed memory map
/// which can't be checked during construction of this type. The iterator
/// might yield unexpected results.  
#[derive(Debug, Clone)]
pub struct MemoryMapIter<'a> {
    pub(crate) memory_map: &'a dyn MemoryMap,
    pub(crate) index: usize,
}

impl<'a> Iterator for MemoryMapIter<'a> {
    type Item = &'a MemoryDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        let desc = self.memory_map.get(self.index)?;

        self.index += 1;

        Some(desc)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let sz = self.memory_map.len() - self.index;

        (sz, Some(sz))
    }
}

impl ExactSizeIterator for MemoryMapIter<'_> {
    fn len(&self) -> usize {
        self.memory_map.len()
    }
}
