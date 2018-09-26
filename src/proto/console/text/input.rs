use core::mem;
use crate::{Event, Result, status, Status};

/// Interface for text-based input devices.
#[repr(C)]
pub struct Input {
    reset: extern "win64" fn(this: &mut Input, extended: bool) -> Status,
    read_key_stroke: extern "win64" fn(this: &mut Input, key: &mut Key) -> Status,
    wait_for_key: Event,
}

impl Input {
    /// Resets the input device hardware.
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

    /// Reads the next keystroke from the input device, if any.
    ///
    /// Use wait_for_key_event() with the BootServices::wait_for_event()
    /// interface in order to wait for a key to be pressed.
    ///
    /// # Errors
    ///
    /// - `DeviceError` if there was an issue with the input device
    pub fn read_key(&mut self) -> Result<Option<Key>> {
        let mut key = unsafe { mem::uninitialized() };

        match (self.read_key_stroke)(self, &mut key) {
            status::SUCCESS => Ok(Some(key)),
            status::NOT_READY => Ok(None),
            error => Err(error),
        }
    }

    /// Event to use with BootServices::wait_for_event() to wait for a key to be
    /// available
    pub fn wait_for_key_event(&self) -> Event {
        self.wait_for_key
    }
}

/// A key read from the console.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct Key {
    /// The key's scan code.
    pub scan_code: ScanCode,
    /// Associated Unicode character,
    /// or 0 if not printable.
    pub unicode_char: u16,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u16)]
#[allow(missing_docs)]
pub enum ScanCode {
    Null,
    /// Move cursor up 1 row.
    Up,
    /// Move cursor down 1 row.
    Down,
    /// Move cursor right 1 column.
    Right,
    /// Move cursor left 1 column.
    Left,
    Home,
    End,
    Insert,
    Delete,
    PageUp,
    PageDown,
    Function1,
    Function2,
    Function3,
    Function4,
    Function5,
    Function6,
    Function7,
    Function8,
    Function9,
    Function10,
    Function11,
    Function12,
    Escape,

    Function13 = 0x68,
    Function14,
    Function15,
    Function16,
    Function17,
    Function18,
    Function19,
    Function20,
    Function21,
    Function22,
    Function23,
    Function24,

    Mute = 0x7F,

    VolumeUp = 0x80,
    VolumeDown,

    BrightnessUp = 0x100,
    BrightnessDown,
    Suspend,
    Hibernate,
    ToggleDisplay,
    Recovery,
    Eject,
}

impl_proto! {
    protocol Input {
        GUID = 0x387477c1, 0x69c7, 0x11d2, [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b];
    }
}
