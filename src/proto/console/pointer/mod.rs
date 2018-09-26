//! Pointer device access.

use core::mem;
use crate::{Event, Result, status, Status};

/// Provides information about a pointer device.
#[repr(C)]
pub struct Pointer {
    reset: extern "win64" fn(this: &mut Pointer, ext_verif: bool) -> Status,
    get_state: extern "win64" fn(this: &Pointer, state: &mut PointerState) -> Status,
    wait_for_input: Event,
    mode: &'static PointerMode,
}

impl Pointer {
    /// Resets the pointer device hardware.
    ///
    /// The `extended_verification` parameter is used to request that UEFI
    /// performs an extended check and reset of the input device.
    ///
    /// # Errors
    ///
    /// - `DeviceError` if the device is malfunctioning and cannot be reset.
    pub fn reset(&mut self, extended_verification: bool) -> Result<()> {
        (self.reset)(self, extended_verification).into()
    }

    /// Retrieves the pointer device's current state, if a state change occured
    /// since the last time this function was called.
    ///
    /// Use wait_for_input_event() with the BootServices::wait_for_event()
    /// interface in order to wait for input from the pointer device.
    ///
    /// # Errors
    /// - `DeviceError` if there was an issue with the pointer device.
    pub fn read_state(&mut self) -> Result<Option<PointerState>> {
        let mut pointer_state = unsafe { mem::uninitialized() };

        match (self.get_state)(self, &mut pointer_state) {
            status::SUCCESS => Ok(Some(pointer_state)),
            status::NOT_READY => Ok(None),
            error => Err(error),
        }
    }

    /// Event to use with BootServices::wait_for_event() to wait for input from
    /// the pointer device
    pub fn wait_for_input_event(&self) -> Event {
        self.wait_for_input
    }

    /// Returns a reference to the pointer device information.
    pub fn mode(&self) -> &PointerMode {
        self.mode
    }
}

impl_proto! {
    protocol Pointer {
        GUID = 0x31878c87, 0xb75, 0x11d5, [0x9a, 0x4f, 0x00, 0x90, 0x27, 0x3f, 0xc1, 0x4d];
    }
}

/// Information about this pointer device.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct PointerMode {
    // The pointer device's resolution on the X/Y/Z axis in counts/mm.
    // If a value is 0, then the device does _not_ support that axis.
    resolution: (u64, u64, u64),
    /// Whether the devices has a left button / right button.
    has_button: (bool, bool),
}

/// The relative change in the pointer's state.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct PointerState {
    /// The relative movement on the X/Y/Z axis.
    ///
    /// If `PointerMode` indicates an axis is not supported, it must be ignored.
    pub relative_movement: (i32, i32, i32),
    /// Whether the left / right mouse button is currently pressed.
    ///
    /// If `PointerMode` indicates a button is not supported, it must be ignored.
    pub button: (bool, bool),
}
