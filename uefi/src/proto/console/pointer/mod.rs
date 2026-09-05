// SPDX-License-Identifier: MIT OR Apache-2.0

//! Pointer device access.

use crate::proto::unsafe_protocol;
use crate::{Error, Event, Result, Status, StatusExt};
use uefi_raw::protocol::console::SimplePointerProtocol;

/// Simple Pointer [`Protocol`]. Provides information about a pointer device.
///
/// Pointer devices are mouses, touchpads, and touchscreens.
///
/// [`Protocol`]: uefi::proto::Protocol
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(SimplePointerProtocol::GUID)]
pub struct Pointer(SimplePointerProtocol);

impl Pointer {
    /// Resets the pointer device hardware.
    ///
    /// # Arguments
    /// The `extended_verification` parameter is used to request that UEFI
    /// performs an extended check and reset of the input device.
    ///
    /// # Errors
    /// - `DeviceError` if the device is malfunctioning and cannot be reset.
    pub fn reset(&mut self, extended_verification: bool) -> Result {
        // SAFETY: We have an exclusive reference to `self`, and `&mut
        // self.0` is a valid protocol pointer.
        unsafe { (self.0.reset)(&mut self.0, extended_verification.into()) }.to_result()
    }

    /// Retrieves the pointer device's current state, if a state change occurred
    /// since the last time this function was called.
    ///
    /// Use `wait_for_input_event()` with the [`boot::wait_for_event`]
    /// interface in order to wait for input from the pointer device.
    ///
    /// # Errors
    /// - `DeviceError` if there was an issue with the pointer device.
    ///
    /// [`boot::wait_for_event`]: crate::boot::wait_for_event
    pub fn read_state(&mut self) -> Result<Option<PointerState>> {
        let mut pointer_state = PointerState::default();
        let pointer_state_ptr: *mut _ = &mut pointer_state;

        // SAFETY: We have an exclusive reference to `self`, `&self.0` is a
        // valid protocol pointer and `pointer_state_ptr` is a valid pointer to
        // stack memory initialized to receive the output.
        match unsafe { (self.0.get_state)(&self.0, pointer_state_ptr.cast()) } {
            Status::NOT_READY => Ok(None),
            other => other.to_result_with_val(|| Some(pointer_state)),
        }
    }

    /// Event to be used with [`boot::wait_for_event`] in order to wait
    /// for input from the pointer device
    ///
    /// [`boot::wait_for_event`]: crate::boot::wait_for_event
    pub fn wait_for_input_event(&self) -> Result<Event> {
        // SAFETY:
        // 1. If null (unsupported), `Event::from_ptr` safely returns `None`.
        // 2. If non-null, the UEFI spec guarantees the driver created a valid `EFI_EVENT`.
        unsafe { Event::from_ptr(self.0.wait_for_input) }.ok_or(Error::from(Status::UNSUPPORTED))
    }

    /// Returns a reference to the pointer device information.
    #[must_use]
    pub const fn mode(&self) -> &PointerMode {
        // SAFETY:
        // 1. `mode` points to valid, initialized memory for the lifetime of the
        //    protocol, matching the lifetime of `&self`.
        // 2. `PointerMode` is `#[repr(C)]` with exact same size, field offsets,
        //    and alignment as `SimplePointerMode`.
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
