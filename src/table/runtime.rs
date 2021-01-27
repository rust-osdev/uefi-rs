//! UEFI services available at runtime, even after the OS boots.

use super::Header;
use crate::table::boot::MemoryDescriptor;
use crate::{Result, Status};
use bitflags::bitflags;
use core::fmt;
use core::mem::MaybeUninit;
use core::ptr;
use uefi_sys::{EFI_MEMORY_DESCRIPTOR, EFI_RUNTIME_SERVICES, EFI_TIME, EFI_TIME_CAPABILITIES};

/// Contains pointers to all of the runtime services.
///
/// This table, and the function pointers it contains are valid
/// even after the UEFI OS loader and OS have taken control of the platform.
#[repr(C)]
pub struct RuntimeServices {
    /// Unsafe raw type extracted from EDK2
    pub raw: EFI_RUNTIME_SERVICES,
}

impl RuntimeServices {
    /// Query the table header
    pub fn header(&self) -> Header {
        Header { raw: self.raw.Hdr }
    }

    /// Query the current time and date information
    pub fn get_time(&self) -> Result<Time> {
        let mut time = MaybeUninit::<Time>::uninit();
        Status::from_raw_api(unsafe {
            self.raw.GetTime.unwrap()(
                time.as_mut_ptr() as *mut Time as *mut EFI_TIME,
                ptr::null_mut(),
            )
        })
        .into_with_val(|| unsafe { time.assume_init() })
    }

    /// Query the current time and date information and the RTC capabilities
    pub fn get_time_and_caps(&self) -> Result<(Time, TimeCapabilities)> {
        let mut time = MaybeUninit::<Time>::uninit();
        let mut caps = MaybeUninit::<TimeCapabilities>::uninit();
        Status::from_raw_api(unsafe {
            self.raw.GetTime.unwrap()(
                time.as_mut_ptr() as *mut Time as *mut EFI_TIME,
                caps.as_mut_ptr() as *mut TimeCapabilities as *mut EFI_TIME_CAPABILITIES,
            )
        })
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
        Status::from_raw_api(self.raw.SetTime.unwrap()(
            time as *const Time as *mut Time as *mut EFI_TIME,
        ))
        .into()
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
        let entry_version = crate::table::boot::EFI_MEMORY_DESCRIPTOR_VERSION;
        let map_ptr = map.as_mut_ptr();
        Status::from_raw_api(self.raw.SetVirtualAddressMap.unwrap()(
            map_size as _,
            entry_size as _,
            entry_version,
            map_ptr as *mut MemoryDescriptor as *mut EFI_MEMORY_DESCRIPTOR,
        ))
        .into()
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

        unsafe {
            self.raw.ResetSystem.unwrap()(
                rt as _,
                status.0 as _,
                size as _,
                data as *mut u8 as *mut core::ffi::c_void,
            )
        }
        panic!("The impossible happened, ResetSystem did not reset the system.")
    }
}

impl super::Table for RuntimeServices {
    const SIGNATURE: u64 = 0x5652_4553_544e_5552;
}

/// The current time information
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Time {
    /// Unsafe raw type extracted from EDK2
    pub raw: EFI_TIME,
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
            raw: EFI_TIME {
                Year: year,
                Month: month,
                Day: day,
                Hour: hour,
                Minute: minute,
                Second: second,
                Pad1: 0,
                Nanosecond: nanosecond,
                TimeZone: time_zone,
                Daylight: daylight.bits(),
                Pad2: 0,
            },
        }
    }

    /// Query the year
    pub fn year(&self) -> u16 {
        self.raw.Year
    }

    /// Query the month
    pub fn month(&self) -> u8 {
        self.raw.Month
    }

    /// Query the day
    pub fn day(&self) -> u8 {
        self.raw.Day
    }

    /// Query the hour
    pub fn hour(&self) -> u8 {
        self.raw.Hour
    }

    /// Query the minute
    pub fn minute(&self) -> u8 {
        self.raw.Minute
    }

    /// Query the second
    pub fn second(&self) -> u8 {
        self.raw.Second
    }

    /// Query the nanosecond
    pub fn nanosecond(&self) -> u32 {
        self.raw.Nanosecond
    }

    /// Query the time offset in minutes from UTC, or None if using local time
    pub fn time_zone(&self) -> Option<i16> {
        if self.raw.TimeZone == 2047 {
            None
        } else {
            Some(self.raw.TimeZone)
        }
    }

    /// Query the daylight savings time information
    pub fn daylight(&self) -> Daylight {
        Daylight {
            bits: self.raw.Daylight,
        }
    }
}

impl fmt::Debug for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}-{} ", self.raw.Year, self.raw.Month, self.raw.Day)?;
        write!(
            f,
            "{}:{}:{}.{} ",
            self.raw.Hour, self.raw.Minute, self.raw.Second, self.raw.Nanosecond
        )?;
        write!(f, "{} {:?}", self.raw.TimeZone, self.raw.Daylight)
    }
}

/// Real time clock capabilities
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct TimeCapabilities {
    /// Unsafe raw type extracted from EDK2
    pub raw: EFI_TIME_CAPABILITIES,
}

impl TimeCapabilities {
    /// Reporting resolution of the clock in counts per second. 1 for a normal
    /// PC-AT CMOS RTC device, which reports the time with 1-second resolution
    pub fn resolution(&self) -> u32 {
        self.raw.Resolution
    }

    /// Timekeeping accuracy in units of 1e-6 parts per million.
    pub fn accuracy(&self) -> u32 {
        self.raw.Accuracy
    }

    /// Whether a time set operation clears the device's time below the
    /// "resolution" reporting level. False for normal PC-AT CMOS RTC devices.
    pub fn sets_to_zero(&self) -> bool {
        self.raw.SetsToZero != 0
    }
}
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
