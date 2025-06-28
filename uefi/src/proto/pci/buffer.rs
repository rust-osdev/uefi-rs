// SPDX-License-Identifier: MIT OR Apache-2.0

//! Defines wrapper allocated by PCI Root Bridge protocol.

use core::cell::RefCell;
use core::fmt::{Debug, Formatter};
use core::mem::{ManuallyDrop, MaybeUninit};
use core::num::NonZeroUsize;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use ghost_cell::{GhostCell, GhostToken};
use log::debug;
use uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol;
use uefi_raw::Status;

/// Smart pointer for wrapping owned buffer allocated by PCI Root Bridge protocol.
pub struct PciBuffer<'b, 'id, T> {
    base: NonNull<T>,
    pages: NonZeroUsize,
    proto: &'b GhostCell<'id, PciRootBridgeIoProtocol>,
    token: &'b RefCell<GhostToken<'id>>
}

impl<'b, 'id, T> PciBuffer<'b, 'id, MaybeUninit<T>> {

    /// Creates wrapper for buffer allocated by PCI Root Bridge protocol.
    /// Passed protocol is stored as a pointer along with its lifetime so that it doesn't
    /// block others from using its mutable functions.
    #[must_use]
    pub const fn new(
        base: NonNull<MaybeUninit<T>>,
        pages: NonZeroUsize,
        proto: &'b GhostCell<'id, PciRootBridgeIoProtocol>,
        token: &'b RefCell<GhostToken<'id>>
    ) -> Self {
        Self {
            base,
            pages,
            proto,
            token,
        }
    }

    /// Assumes the contents of this buffer have been initialized.
    ///
    /// # Safety
    /// Callers of this function must guarantee that value stored is valid.
    #[must_use]
    pub unsafe fn assume_init(self) -> PciBuffer<'b, 'id, T> {
        let old = ManuallyDrop::new(self);
        PciBuffer {
            base: old.base.cast(),
            pages: old.pages,
            proto: old.proto,
            token: old.token,
        }
    }
}

impl<'b, 'id, T> AsRef<T> for PciBuffer<'b, 'id, T> {
    fn as_ref(&self) -> &T {
        unsafe { self.base.as_ref() }
    }
}

impl<'b, 'id, T> AsMut<T> for PciBuffer<'b, 'id, T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { self.base.as_mut() }
    }
}

impl<'b, 'id, T> Deref for PciBuffer<'b, 'id, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'b, 'id, T> DerefMut for PciBuffer<'b, 'id, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<'b, 'id, T> Drop for PciBuffer<'b, 'id, T> {
    fn drop(&mut self) {
        let token = self.token.borrow();
        let protocol = self.proto.borrow(token.deref());
        let status = unsafe {
            (protocol.free_buffer)(protocol, self.pages.get(), self.base.as_ptr().cast())
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

impl<'b, 'id, T> Debug for PciBuffer<'b, 'id, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut debug = f.debug_struct("PciBuffer");
        debug.field("base", &self.base);
        debug.field("pages", &self.pages);

        if let Ok(token) = self.token.try_borrow() {
            let protocol = self.proto.borrow(token.deref());
            debug.field("proto", protocol);
        } else {
            debug.field("proto", &"unavailable");
        };

        debug.finish()
    }
}