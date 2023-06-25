//! Pointer device access.

use crate::proto::unsafe_protocol;
use crate::{Event, Result, Status, StatusExt};
use uefi_raw::protocol::console::SimplePointerProtocol;

/// Provides information about a pointer device.
#[repr(transparent)]
#[unsafe_protocol(SimplePointerProtocol::GUID)]
pub struct Pointer(SimplePointerProtocol);

impl Pointer {
    /// Resets the pointer device hardware.
    ///
    /// The `extended_verification` parameter is used to request that UEFI
    /// performs an extended check and reset of the input device.
    ///
    /// # Errors
    ///
    /// - `DeviceError` if the device is malfunctioning and cannot be reset.
    pub fn reset(&mut self, extended_verification: bool) -> Result {
        unsafe { (self.0.reset)(&mut self.0, extended_verification) }.to_result()
    }

    /// Retrieves the pointer device's current state, if a state change occurred
    /// since the last time this function was called.
    ///
    /// Use `wait_for_input_event()` with the `BootServices::wait_for_event()`
    /// interface in order to wait for input from the pointer device.
    ///
    /// # Errors
    /// - `DeviceError` if there was an issue with the pointer device.
    pub fn read_state(&mut self) -> Result<Option<PointerState>> {
        let mut pointer_state = PointerState::default();
        let pointer_state_ptr: *mut _ = &mut pointer_state;

        match unsafe { (self.0.get_state)(&mut self.0, pointer_state_ptr.cast()) } {
            Status::NOT_READY => Ok(None),
            other => other.to_result_with_val(|| Some(pointer_state)),
        }
    }

    /// Event to be used with `BootServices::wait_for_event()` in order to wait
    /// for input from the pointer device
    #[must_use]
    pub fn wait_for_input_event(&self) -> Option<Event> {
        unsafe { Event::from_ptr(self.0.wait_for_input) }
    }

    /// Returns a reference to the pointer device information.
    #[must_use]
    pub const fn mode(&self) -> &PointerMode {
        unsafe { &*self.0.mode.cast() }
    }
}

/// Information about this pointer device.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct PointerMode {
    /// The pointer device's resolution on the X/Y/Z axis in counts/mm.
    /// If a value is 0, then the device does _not_ support that axis.
    pub resolution: [u64; 3],
    /// Whether the devices has a left button / right button.
    pub has_button: [bool; 2],
}

/// The relative change in the pointer's state.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct PointerState {
    /// The relative movement on the X/Y/Z axis.
    ///
    /// If `PointerMode` indicates an axis is not supported, it must be ignored.
    pub relative_movement: [i32; 3],
    /// Whether the left / right mouse button is currently pressed.
    ///
    /// If `PointerMode` indicates a button is not supported, it must be ignored.
    pub button: [bool; 2],
}
