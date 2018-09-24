use core::mem;
use crate::{Result, Status};

/// Interface for text-based input devices.
#[repr(C)]
pub struct Input {
    reset: extern "win64" fn(this: &mut Input, extended: bool) -> Status,
    read_key_stroke: extern "win64" fn(this: &mut Input, key: &mut Key) -> Status,
}

impl Input {
    /// Resets the input device hardware.
    pub fn reset(&mut self, extended: bool) -> Result<()> {
        (self.reset)(self, extended).into()
    }

    /// Reads the next keystroke from the input device.
    ///
    /// Returns `Err(NotReady)` if no keystroke is available yet.
    pub fn read_key(&mut self) -> Result<Key> {
        let mut key = unsafe { mem::uninitialized() };
        (self.read_key_stroke)(self, &mut key)?;
        Ok(key)
    }

    /// Blocks until a key is read from the device or an error occurs.
    pub fn read_key_sync(&mut self) -> Result<Key> {
        loop {
            match self.read_key() {
                // Received a key, exit loop.
                Ok(key) => return Ok(key),
                Err(code) => {
                    match code {
                        // Wait for key press.
                        Status::NotReady => (),
                        // Exit on error, no point in looping.
                        _ => return Err(code),
                    }
                }
            }
        }
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
