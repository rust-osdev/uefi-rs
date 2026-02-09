// SPDX-License-Identifier: MIT OR Apache-2.0

//! Date and time types.

use bitflags::bitflags;
use core::fmt::{self, Display, Formatter};

/// Generic non-EFI helpers to work with time.
#[allow(unused)]
mod helpers {
    #[inline]
    pub const fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // ---------------------------
        // is_leap_year: basic cases

        #[test]
        fn leap_years_divisible_by_4() {
            assert!(is_leap_year(1996));
            assert!(is_leap_year(2024));
            assert!(is_leap_year(0)); // year 0 is divisible by 400
        }

        #[test]
        fn common_years_not_divisible_by_4() {
            assert!(!is_leap_year(1999));
            assert!(!is_leap_year(2023));
            assert!(!is_leap_year(1));
        }

        #[test]
        fn centuries_not_divisible_by_400_are_not_leap_years() {
            assert!(!is_leap_year(1700));
            assert!(!is_leap_year(1800));
            assert!(!is_leap_year(1900));
            assert!(!is_leap_year(2100));
        }

        #[test]
        fn centuries_divisible_by_400_are_leap_years() {
            assert!(is_leap_year(1600));
            assert!(is_leap_year(2000));
            assert!(is_leap_year(2400));
        }

        // ------------------------------------
        // is_leap_year: negative / boundary

        #[test]
        fn negative_years_follow_same_divisibility_rules() {
            assert!(is_leap_year(-4));
            assert!(!is_leap_year(-1));
            assert!(!is_leap_year(-100));
            assert!(is_leap_year(-400));
        }

        #[test]
        fn i32_boundaries_do_not_overflow() {
            // These tests ensure no arithmetic overflow or panic occurs
            assert!(!is_leap_year(i32::MAX));
            assert!(is_leap_year(i32::MIN)); // i32::MIN % 400 == 0
        }
    }
}

/// Date and time representation.
#[derive(Debug, Default, Copy, Clone, Eq)]
#[repr(C)]
pub struct Time {
    /// Year. Valid range: `1900..=9999`.
    pub year: u16,

    /// Month. Valid range: `1..=12`.
    pub month: u8,

    /// Day of the month. Valid range: `1..=31`.
    pub day: u8,

    /// Hour. Valid range: `0..=23`.
    pub hour: u8,

    /// Minute. Valid range: `0..=59`.
    pub minute: u8,

    /// Second. Valid range: `0..=59`.
    pub second: u8,

    /// Unused padding.
    pub pad1: u8,

    /// Nanosececond. Valid range: `0..=999_999_999`.
    pub nanosecond: u32,

    /// Offset in minutes from UTC. Valid range: `-1440..=1440`, or
    /// [`Time::UNSPECIFIED_TIMEZONE`].
    pub time_zone: i16,

    /// Daylight savings time information.
    pub daylight: Daylight,

    /// Unused padding.
    pub pad2: u8,
}

impl Time {
    /// Indicates the time should be interpreted as local time.
    pub const UNSPECIFIED_TIMEZONE: i16 = 0x07ff;

    /// Creates a new UEFI [`Time`] from UTC components.
    #[must_use]
    pub const fn from_utc_time(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
    ) -> Self {
        // TODO validate() + return result
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            pad1: 0,
            nanosecond,
            time_zone: 0, // UTC
            daylight: Daylight::empty(),
            pad2: 0,
        }
    }

    /// Create an invalid `Time` with all fields set to zero.
    #[must_use]
    pub const fn invalid() -> Self {
        Self {
            year: 0,
            month: 0,
            day: 0,
            hour: 0,
            minute: 0,
            second: 0,
            pad1: 0,
            nanosecond: 0,
            time_zone: 0,
            daylight: Daylight::empty(),
            pad2: 0,
        }
    }

    /// True if all fields are within valid ranges, false otherwise.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        (1900..=9999).contains(&self.year)
            && (1..=12).contains(&self.month)
            && (1..=31).contains(&self.day)
            && self.hour <= 23
            && self.minute <= 59
            && self.second <= 59
            && self.nanosecond <= 999_999_999
            && ((-1440..=1440).contains(&self.time_zone)
                || self.time_zone == Self::UNSPECIFIED_TIMEZONE)
    }
}

impl Display for Time {
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
                write!(f, "UTC+{offset_in_hours}")?;
            }
            // time zones with 30min offset (and perhaps other special time zones)
            else {
                write!(f, "UTC+{offset_in_hours:.1}")?;
            }
        }

        Ok(())
    }
}

/// The padding fields of `Time` are ignored for comparison.
impl PartialEq for Time {
    fn eq(&self, other: &Self) -> bool {
        self.year == other.year
            && self.month == other.month
            && self.day == other.day
            && self.hour == other.hour
            && self.minute == other.minute
            && self.second == other.second
            && self.nanosecond == other.nanosecond
            && self.time_zone == other.time_zone
            && self.daylight == other.daylight
    }
}

bitflags! {
    /// A bitmask containing daylight savings time information.
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Daylight: u8 {
        /// Daylight information available or not applicable to this time
        /// zone.
        const NONE = 0;

        /// Time is affected by daylight savings time.
        const ADJUST_DAYLIGHT = 0x01;

        /// Time has been adjusted for daylight savings time.
        const IN_DAYLIGHT = 0x02;
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_time_display() {
        let mut time = Time {
            year: 2023,
            month: 5,
            day: 18,
            hour: 11,
            minute: 29,
            second: 57,
            nanosecond: 123_456_789,
            time_zone: Time::UNSPECIFIED_TIMEZONE,
            daylight: Daylight::empty(),
            pad1: 0,
            pad2: 0,
        };
        assert_eq!(time.to_string(), "2023-05-18 11:29:57.123456789 (local)");

        time.time_zone = 120;
        assert_eq!(time.to_string(), "2023-05-18 11:29:57.123456789UTC+2");

        time.time_zone = 150;
        assert_eq!(time.to_string(), "2023-05-18 11:29:57.123456789UTC+2.5");
    }
}
