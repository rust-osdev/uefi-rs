use crate::proto::unsafe_protocol;
use crate::{Char16, Event, Result, Status, StatusExt};
use core::mem::MaybeUninit;
use uefi_raw::protocol::console::InputKey;

/// Interface for text-based input devices.
#[repr(C)]
#[unsafe_protocol("387477c1-69c7-11d2-8e39-00a0c969723b")]
pub struct Input {
    reset: extern "efiapi" fn(this: &mut Input, extended: bool) -> Status,
    read_key_stroke: extern "efiapi" fn(this: &mut Input, key: *mut InputKey) -> Status,
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
    pub fn reset(&mut self, extended_verification: bool) -> Result {
        (self.reset)(self, extended_verification).to_result()
    }

    /// Reads the next keystroke from the input device, if any.
    ///
    /// Use [`wait_for_key_event`] with the [`BootServices::wait_for_event`]
    /// interface in order to wait for a key to be pressed.
    ///
    /// [`BootServices::wait_for_event`]: uefi::table::boot::BootServices::wait_for_event
    /// [`wait_for_key_event`]: Self::wait_for_key_event
    ///
    /// # Errors
    ///
    /// - [`Status::DEVICE_ERROR`] if there was an issue with the input device
    ///
    /// # Examples
    ///
    /// ```
    /// use log::info;
    /// use uefi::proto::console::text::{Input, Key, ScanCode};
    /// use uefi::table::boot::BootServices;
    /// use uefi::{Char16, Result, ResultExt};
    ///
    /// fn read_keyboard_events(boot_services: &BootServices, input: &mut Input) -> Result {
    ///     loop {
    ///         // Pause until a keyboard event occurs.
    ///         let mut events = unsafe { [input.wait_for_key_event().unsafe_clone()] };
    ///         boot_services
    ///             .wait_for_event(&mut events)
    ///             .discard_errdata()?;
    ///
    ///         let u_key = Char16::try_from('u').unwrap();
    ///         match input.read_key()? {
    ///             // Example of handling a printable key: print a message when
    ///             // the 'u' key is pressed.
    ///             Some(Key::Printable(key)) if key == u_key => {
    ///                 info!("the 'u' key was pressed");
    ///             }
    ///
    ///             // Example of handling a special key: exit the loop when the
    ///             // escape key is pressed.
    ///             Some(Key::Special(ScanCode::ESCAPE)) => {
    ///                 break;
    ///             }
    ///             _ => {}
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn read_key(&mut self) -> Result<Option<Key>> {
        let mut key = MaybeUninit::<InputKey>::uninit();

        match (self.read_key_stroke)(self, key.as_mut_ptr()) {
            Status::NOT_READY => Ok(None),
            other => other.to_result_with_val(|| Some(unsafe { key.assume_init() }.into())),
        }
    }

    /// Event to be used with `BootServices::wait_for_event()` in order to wait
    /// for a key to be available
    #[must_use]
    pub const fn wait_for_key_event(&self) -> &Event {
        &self.wait_for_key
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

impl From<InputKey> for Key {
    fn from(k: InputKey) -> Key {
        if k.scan_code == ScanCode::NULL.0 {
            Key::Printable(Char16::try_from(k.unicode_char).unwrap())
        } else {
            Key::Special(ScanCode(k.scan_code))
        }
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
