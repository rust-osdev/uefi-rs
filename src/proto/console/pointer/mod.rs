//! Pointer device access.

use crate::proto::Protocol;
use crate::{unsafe_guid, Event, Result, Status};
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use uefi_sys::{EFI_SIMPLE_POINTER_MODE, EFI_SIMPLE_POINTER_PROTOCOL, EFI_SIMPLE_POINTER_STATE};

/// Provides information about a pointer device.
#[repr(C)]
#[unsafe_guid("31878c87-0b75-11d5-9a4f-0090273fc14d")]
#[derive(Protocol)]
pub struct Pointer<'boot> {
    /// Unsafe raw type extracted from EDK2
    pub raw: EFI_SIMPLE_POINTER_PROTOCOL,
    _marker: PhantomData<&'boot ()>,
}

impl<'boot> Pointer<'boot> {
    /// Resets the pointer device hardware.
    ///
    /// The `extended_verification` parameter is used to request that UEFI
    /// performs an extended check and reset of the input device.
    ///
    /// # Errors
    ///
    /// - `DeviceError` if the device is malfunctioning and cannot be reset.
    pub fn reset(&mut self, extended_verification: bool) -> Result {
        Status::from_raw_api(unsafe {
            self.raw.Reset.unwrap()(&mut self.raw, extended_verification as u8)
        })
        .into()
    }

    /// Retrieves the pointer device's current state, if a state change occured
    /// since the last time this function was called.
    ///
    /// Use `wait_for_input_event()` with the `BootServices::wait_for_event()`
    /// interface in order to wait for input from the pointer device.
    ///
    /// # Errors
    /// - `DeviceError` if there was an issue with the pointer device.
    pub fn read_state(&mut self) -> Result<Option<PointerState>> {
        let mut pointer_state = MaybeUninit::<PointerState>::uninit();

        match Status::from_raw_api(unsafe {
            self.raw.GetState.unwrap()(
                &mut self.raw,
                pointer_state.as_mut_ptr() as *mut PointerState as *mut EFI_SIMPLE_POINTER_STATE,
            )
        }) {
            Status::NOT_READY => Ok(None.into()),
            other => other.into_with_val(|| unsafe { Some(pointer_state.assume_init()) }),
        }
    }

    /// Event to be used with `BootServices::wait_for_event()` in order to wait
    /// for input from the pointer device
    pub fn wait_for_input_event(&self) -> Event {
        Event(self.raw.WaitForInput)
    }

    /// Returns a reference to the pointer device information.
    pub fn mode(&self) -> &PointerMode {
        unsafe { &*(self.raw.Mode as *mut EFI_SIMPLE_POINTER_MODE as *mut PointerMode) }
    }
}

/// Information about this pointer device.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct PointerMode {
    /// Unsafe raw type extracted from EDK2
    pub raw: EFI_SIMPLE_POINTER_MODE,
}

impl PointerMode {
    /// The pointer device's resolution on the X/Y/Z axis in counts/mm.
    /// If a value is 0, then the device does _not_ support that axis.
    pub fn resolution(&self) -> (u64, u64, u64) {
        (
            self.raw.ResolutionX,
            self.raw.ResolutionY,
            self.raw.ResolutionZ,
        )
    }

    /// Whether the devices has a left button / right button.
    pub fn has_button(&self) -> (bool, bool) {
        (self.raw.LeftButton != 0, self.raw.RightButton != 0)
    }
}

/// The relative change in the pointer's state.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct PointerState {
    /// Unsafe raw type extracted from EDK2
    pub raw: EFI_SIMPLE_POINTER_STATE,
}

impl PointerState {
    /// The relative movement on the X/Y/Z axis.
    ///
    /// If `PointerMode` indicates an axis is not supported, it must be ignored.
    pub fn relative_movement(&self) -> (i32, i32, i32) {
        (
            self.raw.RelativeMovementX,
            self.raw.RelativeMovementY,
            self.raw.RelativeMovementZ,
        )
    }
    /// Whether the left / right mouse button is currently pressed.
    ///
    /// If `PointerMode` indicates a button is not supported, it must be ignored.
    pub fn button(&self) -> (bool, bool) {
        (self.raw.LeftButton != 0, self.raw.RightButton != 0)
    }
}
