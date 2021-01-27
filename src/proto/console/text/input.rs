use crate::proto::Protocol;
use crate::{unsafe_guid, Char16, Event, Result, Status};
use core::mem::MaybeUninit;
use uefi_sys::{EFI_INPUT_KEY, EFI_SIMPLE_TEXT_INPUT_PROTOCOL};

/// Interface for text-based input devices.
#[repr(C)]
#[unsafe_guid("387477c1-69c7-11d2-8e39-00a0c969723b")]
#[derive(Protocol)]
pub struct Input {
    /// Unsafe raw type extracted from EDK2
    pub raw: EFI_SIMPLE_TEXT_INPUT_PROTOCOL,
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
    pub fn reset(&mut self, extended_verification: bool) -> Result {
        Status::from_raw_api(unsafe {
            self.raw.Reset.unwrap()(&mut self.raw, extended_verification as u8)
        })
        .into()
    }

    /// Reads the next keystroke from the input device, if any.
    ///
    /// Use `wait_for_key_event()` with the `BootServices::wait_for_event()`
    /// interface in order to wait for a key to be pressed.
    ///
    /// # Errors
    ///
    /// - `DeviceError` if there was an issue with the input device
    pub fn read_key(&mut self) -> Result<Option<Key>> {
        let mut key = MaybeUninit::<RawKey>::uninit();

        match Status::from_raw_api(unsafe {
            self.raw.ReadKeyStroke.unwrap()(
                &mut self.raw,
                key.as_mut_ptr() as *mut RawKey as *mut EFI_INPUT_KEY,
            )
        }) {
            Status::NOT_READY => Ok(None.into()),
            other => other.into_with_val(|| Some(unsafe { key.assume_init() }.into())),
        }
    }

    /// Event to be used with `BootServices::wait_for_event()` in order to wait
    /// for a key to be available
    pub fn wait_for_key_event(&self) -> Event {
        Event(self.raw.WaitForKey)
    }
}

/// A key read from the console (high-level version)
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Key {
    /// The key is associated with a printable Unicode character
    Printable(Char16),

    /// The key is special (arrow, function, multimedia...)
    Special(ScanCode),
}

impl From<RawKey> for Key {
    fn from(k: RawKey) -> Key {
        if k.scan_code() == ScanCode::NULL {
            Key::Printable(k.unicode_char())
        } else {
            Key::Special(k.scan_code())
        }
    }
}

/// A key read from the console (UEFI version)
#[repr(C)]
pub struct RawKey {
    pub raw: EFI_INPUT_KEY,
}

impl RawKey {
    /// The key's scan code.
    /// or 0 if printable
    pub fn scan_code(&self) -> ScanCode {
        ScanCode(self.raw.ScanCode)
    }

    /// Associated Unicode character,
    /// or 0 if not printable.
    pub fn unicode_char(&self) -> Char16 {
        Char16(self.raw.UnicodeChar)
    }
}

newtype_enum! {
/// A keyboard scan code
///
/// Codes 0x8000 -> 0xFFFF are reserved for future OEM extensibility, therefore
/// this C enum is _not_ safe to model as a Rust enum (where the compiler must
/// know about all variants at compile time).
pub enum ScanCode: u16 => #[allow(missing_docs)] {
    /// Null scan code, indicates that the Unicode character should be used.
    NULL        = 0x00,
    /// Move cursor up 1 row.
    UP          = 0x01,
    /// Move cursor down 1 row.
    DOWN        = 0x02,
    /// Move cursor right 1 column.
    RIGHT       = 0x03,
    /// Move cursor left 1 column.
    LEFT        = 0x04,
    HOME        = 0x05,
    END         = 0x06,
    INSERT      = 0x07,
    DELETE      = 0x08,
    PAGE_UP     = 0x09,
    PAGE_DOWN   = 0x0A,
    FUNCTION_1  = 0x0B,
    FUNCTION_2  = 0x0C,
    FUNCTION_3  = 0x0D,
    FUNCTION_4  = 0x0E,
    FUNCTION_5  = 0x0F,
    FUNCTION_6  = 0x10,
    FUNCTION_7  = 0x11,
    FUNCTION_8  = 0x12,
    FUNCTION_9  = 0x13,
    FUNCTION_10 = 0x14,
    FUNCTION_11 = 0x15,
    FUNCTION_12 = 0x16,
    ESCAPE      = 0x17,

    FUNCTION_13 = 0x68,
    FUNCTION_14 = 0x69,
    FUNCTION_15 = 0x6A,
    FUNCTION_16 = 0x6B,
    FUNCTION_17 = 0x6C,
    FUNCTION_18 = 0x6D,
    FUNCTION_19 = 0x6E,
    FUNCTION_20 = 0x6F,
    FUNCTION_21 = 0x70,
    FUNCTION_22 = 0x71,
    FUNCTION_23 = 0x72,
    FUNCTION_24 = 0x73,

    MUTE        = 0x7F,
    VOLUME_UP   = 0x80,
    VOLUME_DOWN = 0x81,

    BRIGHTNESS_UP   = 0x100,
    BRIGHTNESS_DOWN = 0x101,
    SUSPEND         = 0x102,
    HIBERNATE       = 0x103,
    TOGGLE_DISPLAY  = 0x104,
    RECOVERY        = 0x105,
    EJECT           = 0x106,
}}
