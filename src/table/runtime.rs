//! UEFI services available at runtime, even after the OS boots.

use super::Header;
use crate::result::Error;
use crate::table::boot::MemoryDescriptor;
use crate::{CStr16, Char16, Guid, Result, Status};
use bitflags::bitflags;
use core::fmt;
use core::fmt::Formatter;
use core::mem::MaybeUninit;
use core::ptr;

/// Contains pointers to all of the runtime services.
///
/// This table, and the function pointers it contains are valid
/// even after the UEFI OS loader and OS have taken control of the platform.
#[repr(C)]
pub struct RuntimeServices {
    header: Header,
    get_time:
        unsafe extern "efiapi" fn(time: *mut Time, capabilities: *mut TimeCapabilities) -> Status,
    set_time: unsafe extern "efiapi" fn(time: &Time) -> Status,
    // Skip some useless functions.
    _pad: [usize; 2],
    set_virtual_address_map: unsafe extern "efiapi" fn(
        map_size: usize,
        desc_size: usize,
        desc_version: u32,
        virtual_map: *mut MemoryDescriptor,
    ) -> Status,
    _pad2: usize,
    get_variable: unsafe extern "efiapi" fn(
        variable_name: *const Char16,
        vendor_guid: *const Guid,
        attributes: *mut VariableAttributes,
        data_size: *mut usize,
        data: *mut u8,
    ) -> Status,
    get_next_variable_name: unsafe extern "efiapi" fn(
        variable_name_size: *mut usize,
        variable_name: *mut u16,
        vendor_guid: *mut Guid,
    ) -> Status,
    set_variable: unsafe extern "efiapi" fn(
        variable_name: *const Char16,
        vendor_guid: *const Guid,
        attributes: VariableAttributes,
        data_size: usize,
        data: *const u8,
    ) -> Status,
    _pad3: usize,
    reset: unsafe extern "efiapi" fn(
        rt: ResetType,

        status: Status,
        data_size: usize,
        data: *const u8,
    ) -> !,
}

impl RuntimeServices {
    /// Query the current time and date information
    pub fn get_time(&self) -> Result<Time> {
        let mut time = MaybeUninit::<Time>::uninit();
        unsafe { (self.get_time)(time.as_mut_ptr(), ptr::null_mut()) }
            .into_with_val(|| unsafe { time.assume_init() })
    }

    /// Query the current time and date information and the RTC capabilities
    pub fn get_time_and_caps(&self) -> Result<(Time, TimeCapabilities)> {
        let mut time = MaybeUninit::<Time>::uninit();
        let mut caps = MaybeUninit::<TimeCapabilities>::uninit();
        unsafe { (self.get_time)(time.as_mut_ptr(), caps.as_mut_ptr()) }
            .into_with_val(|| unsafe { (time.assume_init(), caps.assume_init()) })
    }

    /// Sets the current local time and date information
    ///
    /// During runtime, if a PC-AT CMOS device is present in the platform, the
    /// caller must synchronize access to the device before calling `set_time`.
    ///
    /// # Safety
    ///
    /// Undefined behavior could happen if multiple tasks try to
    /// use this function at the same time without synchronisation.
    pub unsafe fn set_time(&mut self, time: &Time) -> Result {
        (self.set_time)(time).into()
    }

    /// Changes the runtime addressing mode of EFI firmware from physical to virtual.
    ///
    /// # Safety
    ///
    /// Setting new virtual memory map is unsafe and may cause undefined behaviors.
    pub unsafe fn set_virtual_address_map(&self, map: &mut [MemoryDescriptor]) -> Result {
        // Unsafe Code Guidelines guarantees that there is no padding in an array or a slice
        // between its elements if the element type is `repr(C)`, which is our case.
        //
        // See https://rust-lang.github.io/unsafe-code-guidelines/layout/arrays-and-slices.html
        let map_size = core::mem::size_of_val(map);
        let entry_size = core::mem::size_of::<MemoryDescriptor>();
        let entry_version = crate::table::boot::MEMORY_DESCRIPTOR_VERSION;
        let map_ptr = map.as_mut_ptr();
        (self.set_virtual_address_map)(map_size, entry_size, entry_version, map_ptr).into()
    }

    /// Get the size (in bytes) of a variable. This can be used to find out how
    /// big of a buffer should be passed in to `get_variable`.
    pub fn get_variable_size(&self, name: &CStr16, vendor: &Guid) -> Result<usize> {
        let mut data_size = 0;
        let status = unsafe {
            (self.get_variable)(
                name.as_ptr(),
                vendor,
                ptr::null_mut(),
                &mut data_size,
                ptr::null_mut(),
            )
        };

        if status == Status::BUFFER_TOO_SMALL {
            Status::SUCCESS.into_with_val(|| data_size)
        } else {
            Err(Error::from(status))
        }
    }

    /// Get the contents and attributes of a variable. The size of `buf` must
    /// be at least as big as the variable's size, although it can be
    /// larger. If it is too small, `BUFFER_TOO_SMALL` is returned.
    ///
    /// On success, a tuple containing the variable's value (a slice of `buf`)
    /// and the variable's attributes is returned.
    pub fn get_variable<'a>(
        &self,
        name: &CStr16,
        vendor: &Guid,
        buf: &'a mut [u8],
    ) -> Result<(&'a [u8], VariableAttributes)> {
        let mut attributes = VariableAttributes::empty();
        let mut data_size = buf.len();
        unsafe {
            (self.get_variable)(
                name.as_ptr(),
                vendor,
                &mut attributes,
                &mut data_size,
                buf.as_mut_ptr(),
            )
            .into_with_val(move || (&buf[..data_size], attributes))
        }
    }

    /// Set the value of a variable. This can be used to create a new variable,
    /// update an existing variable, or (when the size of `data` is zero)
    /// delete a variable.
    pub fn set_variable(
        &self,
        name: &CStr16,
        vendor: &Guid,
        attributes: VariableAttributes,
        data: &[u8],
    ) -> Result {
        unsafe {
            (self.set_variable)(name.as_ptr(), vendor, attributes, data.len(), data.as_ptr()).into()
        }
    }

    /// Resets the computer.
    pub fn reset(&self, rt: ResetType, status: Status, data: Option<&[u8]>) -> ! {
        let (size, data) = match data {
            // FIXME: The UEFI spec states that the data must start with a NUL-
            //        terminated string, which we should check... but it does not
            //        specify if that string should be Latin-1 or UCS-2!
            //
            //        PlatformSpecific resets should also insert a GUID after the
            //        NUL-terminated string.
            Some(data) => (data.len(), data.as_ptr()),
            None => (0, ptr::null()),
        };

        unsafe { (self.reset)(rt, status, size, data) }
    }
}

impl super::Table for RuntimeServices {
    const SIGNATURE: u64 = 0x5652_4553_544e_5552;
}

/// The current time information
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Time {
    year: u16,  // 1900 - 9999
    month: u8,  // 1 - 12
    day: u8,    // 1 - 31
    hour: u8,   // 0 - 23
    minute: u8, // 0 - 59
    second: u8, // 0 - 59
    _pad1: u8,
    nanosecond: u32, // 0 - 999_999_999
    time_zone: i16,  // -1440 to 1440, or 2047 if unspecified
    daylight: Daylight,
    _pad2: u8,
}

bitflags! {
    /// Flags describing the capabilities of a memory range.
    pub struct Daylight: u8 {
        /// Time is affected by daylight savings time
        const ADJUST_DAYLIGHT = 0x01;
        /// Time has been adjusted for daylight savings time
        const IN_DAYLIGHT = 0x02;
    }
}

impl Time {
    /// Unspecified Timezone/local time.
    const UNSPECIFIED_TIMEZONE: i16 = 0x07ff;

    /// Build an UEFI time struct
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
        time_zone: i16,
        daylight: Daylight,
    ) -> Self {
        assert!((1900..=9999).contains(&year));
        assert!((1..=12).contains(&month));
        assert!((1..=31).contains(&day));
        assert!(hour <= 23);
        assert!(minute <= 59);
        assert!(second <= 59);
        assert!(nanosecond <= 999_999_999);
        assert!((time_zone >= -1440 && time_zone <= 1440) || time_zone == 2047);
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            _pad1: 0,
            nanosecond,
            time_zone,
            daylight,
            _pad2: 0,
        }
    }

    /// Query the year
    pub fn year(&self) -> u16 {
        self.year
    }

    /// Query the month
    pub fn month(&self) -> u8 {
        self.month
    }

    /// Query the day
    pub fn day(&self) -> u8 {
        self.day
    }

    /// Query the hour
    pub fn hour(&self) -> u8 {
        self.hour
    }

    /// Query the minute
    pub fn minute(&self) -> u8 {
        self.minute
    }

    /// Query the second
    pub fn second(&self) -> u8 {
        self.second
    }

    /// Query the nanosecond
    pub fn nanosecond(&self) -> u32 {
        self.nanosecond
    }

    /// Query the time offset in minutes from UTC, or None if using local time
    pub fn time_zone(&self) -> Option<i16> {
        if self.time_zone == 2047 {
            None
        } else {
            Some(self.time_zone)
        }
    }

    /// Query the daylight savings time information
    pub fn daylight(&self) -> Daylight {
        self.daylight
    }
}

impl fmt::Debug for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02} ", self.year, self.month, self.day)?;
        write!(
            f,
            "{:02}:{:02}:{:02}.{:09}",
            self.hour, self.minute, self.second, self.nanosecond
        )?;
        if self.time_zone == Self::UNSPECIFIED_TIMEZONE {
            write!(f, ", Timezone=local")?;
        } else {
            write!(f, ", Timezone={}", self.time_zone)?;
        }
        write!(f, ", Daylight={:?}", self.daylight)
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02} ", self.year, self.month, self.day)?;
        write!(
            f,
            "{:02}:{:02}:{:02}.{:09}",
            self.hour, self.minute, self.second, self.nanosecond
        )?;

        if self.time_zone == Self::UNSPECIFIED_TIMEZONE {
            write!(f, " (local)")?;
        } else {
            let offset_in_hours = self.time_zone as f32 / 60.0;
            let integer_part = offset_in_hours as i16;
            // We can't use "offset_in_hours.fract()" because it is part of `std`.
            let fraction_part = offset_in_hours - (integer_part as f32);
            // most time zones
            if fraction_part == 0.0 {
                write!(f, "UTC+{}", offset_in_hours)?;
            }
            // time zones with 30min offset (and perhaps other special time zones)
            else {
                write!(f, "UTC+{:.1}", offset_in_hours)?;
            }
        }

        Ok(())
    }
}

/// Real time clock capabilities
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct TimeCapabilities {
    /// Reporting resolution of the clock in counts per second. 1 for a normal
    /// PC-AT CMOS RTC device, which reports the time with 1-second resolution.
    pub resolution: u32,

    /// Timekeeping accuracy in units of 1e-6 parts per million.
    pub accuracy: u32,

    /// Whether a time set operation clears the device's time below the
    /// "resolution" reporting level. False for normal PC-AT CMOS RTC devices.
    pub sets_to_zero: bool,
}

bitflags! {
    /// Flags describing the attributes of a variable.
    pub struct VariableAttributes: u32 {
        /// Variable is maintained across a power cycle.
        const NON_VOLATILE = 0x01;

        /// Variable is accessible during the time that boot services are
        /// accessible.
        const BOOTSERVICE_ACCESS = 0x02;

        /// Variable is accessible during the time that runtime services are
        /// accessible.
        const RUNTIME_ACCESS = 0x04;

        /// Variable is stored in the portion of NVR allocated for error
        /// records.
        const HARDWARE_ERROR_RECORD = 0x08;

        /// Deprecated.
        const AUTHENTICATED_WRITE_ACCESS = 0x10;

        /// Variable payload begins with an EFI_VARIABLE_AUTHENTICATION_2
        /// structure.
        const TIME_BASED_AUTHENTICATED_WRITE_ACCESS = 0x20;

        /// This is never set in the attributes returned by
        /// `get_variable`. When passed to `set_variable`, the variable payload
        /// will be appended to the current value of the variable if supported
        /// by the firmware.
        const APPEND_WRITE = 0x40;

        /// Variable payload begins with an EFI_VARIABLE_AUTHENTICATION_3
        /// structure.
        const ENHANCED_AUTHENTICATED_ACCESS = 0x80;
    }
}

/// Vendor GUID used to access global variables.
pub const GLOBAL_VARIABLE: Guid = Guid::from_values(
    0x8be4df61,
    0x93ca,
    0x11d2,
    0xaa0d,
    [0x00, 0xe0, 0x98, 0x03, 0x2b, 0x8c],
);

/// The type of system reset.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum ResetType {
    /// Resets all the internal circuitry to its initial state.
    ///
    /// This is analogous to power cycling the device.
    Cold = 0,
    /// The processor is reset to its initial state.
    Warm,
    /// The components are powered off.
    Shutdown,
    /// A platform-specific reset type.
    ///
    /// The additional data must be a pointer to
    /// a null-terminated string followed by an UUID.
    PlatformSpecific,
    // SAFETY: This enum is never exposed to the user, but only fed as input to
    //         the firmware. Therefore, unexpected values can never come from
    //         the firmware, and modeling this as a Rust enum seems safe.
}
