// SPDX-License-Identifier: MIT OR Apache-2.0

//! Defines wrapper for pages allocated by PCI Root Bridge protocol.
use core::fmt::Debug;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::num::NonZeroUsize;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use log::{error, trace};
use uefi_raw::Status;
use uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol;

/// Smart pointer for wrapping owned pages allocated by PCI Root Bridge protocol.
///
/// # Lifetime
/// `'p` is the lifetime for Protocol.
#[derive(Debug)]
pub struct PciBuffer<'p, T> {
    pub(super) base: NonNull<T>,
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
    /// Returns the base address of this buffer
    #[must_use]
    pub fn base(&self) -> NonZeroUsize {
        self.base.addr()
    }

    /// Returns the number of pages this buffer uses
    #[must_use]
    pub const fn pages(&self) -> NonZeroUsize {
        self.pages
    }
}

impl<T> AsRef<T> for PciBuffer<'_, T> {
    fn as_ref(&self) -> &T {
        unsafe { self.base.as_ref() }
    }
}

impl<T> AsMut<T> for PciBuffer<'_, T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { self.base.as_mut() }
    }
}

impl<T> Deref for PciBuffer<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for PciBuffer<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T> Drop for PciBuffer<'_, T> {
    fn drop(&mut self) {
        let status = unsafe {
            (self.proto.free_buffer)(self.proto, self.pages.get(), self.base.as_ptr().cast())
        };
        match status {
            Status::SUCCESS => {
                trace!(
                    "Freed {} pages at 0x{:X}",
                    self.pages.get(),
                    self.base.as_ptr().addr()
                );
            }
            Status::INVALID_PARAMETER => {
                error!("PciBuffer was not created through valid protocol usage!")
            }
            etc => {
                error!("Failed to free PciBuffer: {:?}", etc);
            }
        }
    }
}
