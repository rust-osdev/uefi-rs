// SPDX-License-Identifier: MIT OR Apache-2.0

//! Defines wrapper for pages allocated by PCI Root Bridge protocol.

use crate::StatusExt;
use core::cell::UnsafeCell;
use core::fmt::Debug;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::num::NonZeroUsize;
use core::ptr::NonNull;
use log::{error, trace};
use uefi_raw::Status;
use uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol;
use uefi_raw::table::boot::PAGE_SIZE;

/// Smart pointer for wrapping owned pages allocated by PCI Root Bridge protocol.
/// Value stored in this buffer maybe modified by a PCI device.
///
/// # Lifetime
/// `'p` is the lifetime for Protocol.
///
/// # Invariant
/// * Value stored in this memory cannot have a larger alignment requirement
///   than page size, which is 4096.
/// * Value stored in this memory cannot be larger than the buffer's size, which is 4096 * `pages`
#[derive(Debug)]
pub struct PciBuffer<'p, T> {
    pub(super) base: NonNull<UnsafeCell<T>>,
    pub(super) pages: NonZeroUsize,
    pub(super) proto: &'p PciRootBridgeIoProtocol,
}

impl<'p, T> PciBuffer<'p, MaybeUninit<T>> {
    /// Assumes the contents of this buffer have been initialized.
    ///
    /// # Safety
    /// Callers of this function must guarantee that the value stored is valid.
    #[must_use]
    pub const unsafe fn assume_init(self) -> PciBuffer<'p, T> {
        let initialized = PciBuffer {
            base: self.base.cast(),
            pages: self.pages,
            proto: self.proto,
        };
        let _ = ManuallyDrop::new(self);
        initialized
    }
}

impl<'p, T> PciBuffer<'p, T> {
    /// Returns the base pointer of this buffer
    #[must_use]
    pub const fn base_ptr(&self) -> *mut T {
        self.base.as_ptr().cast()
    }

    /// Returns the number of pages this buffer uses
    #[must_use]
    pub const fn pages(&self) -> NonZeroUsize {
        self.pages
    }

    /// Returns the size of this buffer in bytes
    #[must_use]
    pub const fn bytes_size(&self) -> NonZeroUsize {
        self.pages
            .checked_mul(NonZeroUsize::new(PAGE_SIZE).unwrap())
            .expect("Memory size Overflow")
    }

    /// Frees underlying memory of this buffer.
    /// It is recommended to use this over drop implementation.
    pub fn free(self) -> crate::Result {
        self.free_inner()
    }

    fn free_inner(&self) -> crate::Result {
        unsafe { (self.proto.free_buffer)(self.proto, self.pages.get(), self.base.as_ptr().cast()) }
            .to_result_with_val(|| {
                trace!(
                    "Freed {} pages at 0x{:X}",
                    self.pages,
                    self.base.as_ptr().addr()
                )
            })
    }
}

impl<T> Drop for PciBuffer<'_, T> {
    fn drop(&mut self) {
        let Err(status) = self.free_inner() else {
            return;
        };
        match status.status() {
            Status::SUCCESS => {}
            Status::INVALID_PARAMETER => {
                error!("PciBuffer was not created through valid protocol usage!")
            }
            etc => {
                error!("Unexpected error occurred when freeing memory: {:?}", etc);
            }
        }
    }
}
