// SPDX-License-Identifier: MIT OR Apache-2.0

//! Pointer device access.

pub use uefi_raw::protocol::console::{
    SimplePointerMode as PointerMode, SimplePointerState as PointerState,
};

use crate::proto::unsafe_protocol;
use crate::{Error, Event, Result, Status, StatusExt};
use uefi_raw::protocol::console::{
    AbsolutePointerMode, AbsolutePointerProtocol, AbsolutePointerState, SimplePointerProtocol,
};

/// Simple Pointer [`Protocol`]. Provides information about a pointer device.
///
/// Pointer devices are report relative movement, such as mice. For absolute
/// pointing devices, such as touchscreens, see [`AbsolutePointer`].
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
        let mut state = PointerState::default();

        // SAFETY: We have an exclusive reference to `self`, `&mut self.0` is a
        // valid protocol pointer and `&mut state` is a valid pointer to stack
        // memory initialized to receive the output.
        match unsafe { (self.0.get_state)(&mut self.0, &mut state) } {
            Status::NOT_READY => Ok(None),
            other => other.to_result_with_val(|| Some(state)),
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
        // SAFETY: `mode` points to valid, initialized memory for the lifetime
        // of the protocol, matching the lifetime of `&self`.
        unsafe { &*self.0.mode }
    }
}

/// Absolute Pointer [`Protocol`]. Provides coordinate pointing (e.g. touchscreens, tablets).
///
/// For relative pointing devices such as mice, see [`Pointer`].
///
/// [`Protocol`]: uefi::proto::Protocol
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(AbsolutePointerProtocol::GUID)]
pub struct AbsolutePointer(AbsolutePointerProtocol);

impl AbsolutePointer {
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
    /// Use `wait_for_input_event()` with the [`Self::wait_for_input_event`]
    /// interface in order to wait for input from the pointer device.
    ///
    /// # Errors
    /// - `DeviceError` if there was an issue with the pointer device.
    ///
    /// [`boot::wait_for_event`]: crate::boot::wait_for_event
    pub fn read_state(&mut self) -> Result<Option<AbsolutePointerState>> {
        let mut state = AbsolutePointerState::default();
        let state_ptr: *mut _ = &mut state;

        // SAFETY: We have an exclusive reference to `self`, `&self.0` is a
        // valid protocol pointer and `pointer_state_ptr` is a valid pointer to
        // stack memory initialized to receive the output.
        match unsafe { (self.0.get_state)(&mut self.0, state_ptr) } {
            Status::NOT_READY => Ok(None),
            other => other.to_result_with_val(|| Some(state)),
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
    pub const fn mode(&self) -> &AbsolutePointerMode {
        // SAFETY: `mode` points to valid, initialized memory for the lifetime
        // of the protocol, matching the lifetime of `&self`.
        unsafe { &*self.0.mode }
    }
}
