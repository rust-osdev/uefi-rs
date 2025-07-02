// SPDX-License-Identifier: MIT OR Apache-2.0

//! Defines wrapper allocated by PCI Root Bridge protocol.

use core::fmt::Debug;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::num::NonZeroUsize;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use log::debug;
use uefi_raw::Status;
use uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol;

/// Smart pointer for wrapping owned buffer allocated by PCI Root Bridge protocol.
#[derive(Debug)]
pub struct PciBuffer<'p, T> {
    base: NonNull<T>,
    pages: NonZeroUsize,
    proto: &'p PciRootBridgeIoProtocol,
}

impl<'p, T> PciBuffer<'p, MaybeUninit<T>> {
    /// Creates wrapper for buffer allocated by PCI Root Bridge protocol.
    /// Passed protocol is stored as a pointer along with its lifetime so that it doesn't
    /// block others from using its mutable functions.
    #[must_use]
    pub const fn new(
        base: NonNull<MaybeUninit<T>>,
        pages: NonZeroUsize,
        proto: &'p PciRootBridgeIoProtocol,
    ) -> Self {
        Self { base, pages, proto }
    }

    /// Assumes the contents of this buffer have been initialized.
    ///
    /// # Safety
    /// Callers of this function must guarantee that value stored is valid.
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
                debug!(
                    "Freed {} pages at 0x{:X}",
                    self.pages.get(),
                    self.base.as_ptr().addr()
                );
            }
            Status::INVALID_PARAMETER => {
                panic!("PciBuffer was not created through valid protocol usage!")
            }
            _ => unreachable!(),
        }
    }
}
