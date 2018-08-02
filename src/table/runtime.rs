//! UEFI services available at runtime, even after the OS boots.

use crate::Status;
use super::Header;
use core::ptr;

/// Contains pointers to all of the runtime services.
///
/// This table, and the function pointers it contains are valid
/// even after the UEFI OS loader and OS have taken control of the platform.
#[repr(C)]
pub struct RuntimeServices {
    header: Header,
    // Skip some useless functions.
    _pad: [usize; 10],
    reset: extern "C" fn(u32, Status, usize, *const u8) -> !,
}

impl RuntimeServices {
    /// Resets the computer.
    pub fn reset(&self, rt: ResetType, status: Status, data: Option<&[u8]>) -> ! {
        let (size, data) = match data {
            Some(data) => (data.len(), data.as_ptr()),
            None => (0, ptr::null()),
        };

        (self.reset)(rt as u32, status, size, data)
    }
}

impl super::Table for RuntimeServices {
    const SIGNATURE: u64 = 0x5652_4553_544e_5552;
}

/// The type of system reset.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum ResetType {
    /// Resets all the internal circuitry to its initial state.
    ///
    /// This is analogous to power cycling the device.
    Cold,
    /// The processor is reset to its initial state.
    Warm,
    /// The components are powered off.
    Shutdown,
    /// A platform-specific reset type.
    ///
    /// The additional data must be a pointer to
    /// a null-terminated string followed by an UUID.
    PlatformSpecific,
}
