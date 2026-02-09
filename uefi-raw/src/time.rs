// SPDX-License-Identifier: MIT OR Apache-2.0

//! Date and time types.
//!
//! See [`Time`].

use crate::time::helpers::{
    NANOS_PER_SECOND, SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE, days_in_month,
    days_since_unix_epoch, is_leap_year,
};
use bitflags::bitflags;
use core::cmp::Ordering;
use core::fmt::{self, Display, Formatter};
use core::time::Duration;

/// Generic non-EFI helpers to work with time.
mod helpers {
    pub const NANOS_PER_SECOND: i128 = 1_000_000_000;
    pub const SECONDS_PER_MINUTE: i64 = 60;
    pub const SECONDS_PER_HOUR: i64 = 60 * SECONDS_PER_MINUTE;
    pub const SECONDS_PER_DAY: i64 = 24 * SECONDS_PER_HOUR;

    #[inline]
    pub const fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    #[inline]
    pub fn days_in_month(year: i32, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if is_leap_year(year) => 29,
            2 => 28,
            v => panic!("invalid month: {v}"),
        }
    }

    /// Days since Unix epoch (1970-01-01), ignoring time-of-day.
    pub fn days_since_unix_epoch(year: i32, month: u8, day: u8) -> i64 {
        let mut days = 0i64;

        for y in 1970..year {
            days += if is_leap_year(y) { 366 } else { 365 };
        }

        for m in 1..month {
            days += days_in_month(year, m) as i64;
        }

        days + (day as i64 - 1)
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

        // ---------------------------
        // days_in_month: fixed months

        #[test]
        fn months_with_31_days() {
            let months = [1, 3, 5, 7, 8, 10, 12];
            for &m in &months {
                assert_eq!(days_in_month(2023, m), 31);
                assert_eq!(days_in_month(2024, m), 31); // leap year should not matter
            }
        }

        #[test]
        fn months_with_30_days() {
            let months = [4, 6, 9, 11];
            for &m in &months {
                assert_eq!(days_in_month(2023, m), 30);
                assert_eq!(days_in_month(2024, m), 30);
            }
        }

        // ---------------------------
        // days_in_month: February

        #[test]
        fn february_in_common_year() {
            assert_eq!(days_in_month(2023, 2), 28);
            assert_eq!(days_in_month(1900, 2), 28); // century common year
        }

        #[test]
        fn february_in_leap_year() {
            assert_eq!(days_in_month(2024, 2), 29);
            assert_eq!(days_in_month(2000, 2), 29); // century leap year
        }

        // --------------------------------
        // days_in_month: invalid months

        #[test]
        #[should_panic(expected = "invalid month")]
        fn month_zero_panics() {
            days_in_month(2024, 0);
        }

        #[test]
        #[should_panic(expected = "invalid month")]
        fn month_above_12_panics() {
            days_in_month(2024, 13);
        }

        #[test]
        #[should_panic(expected = "invalid month")]
        fn max_u8_month_panics() {
            days_in_month(2024, u8::MAX);
        }

        // --------------------------------
        // days_since_unix_epoch:

        #[test]
        fn test_epoch_start() {
            // January 1, 1970 should be exactly 0 days
            assert_eq!(days_since_unix_epoch(1970, 1, 1), 0);
        }

        #[test]
        fn test_first_year_completion() {
            // December 31, 1970 (365 days in a non-leap year, so 364 days since Jan 1)
            assert_eq!(days_since_unix_epoch(1970, 12, 31), 364);
        }

        #[test]
        fn test_one_full_year() {
            // January 1, 1971 should be 365 days after the epoch
            assert_eq!(days_since_unix_epoch(1971, 1, 1), 365);
        }

        #[test]
        fn test_leap_year_handling() {
            // 1972 was a leap year.
            // Days: 1970 (365) + 1971 (365) = 730
            assert_eq!(days_since_unix_epoch(1972, 1, 1), 730);

            // After February 29, 1972
            // Jan (31) + Feb (29) = 60. So March 1st is day 60 of that year.
            // Total: 730 + 60 = 790
            assert_eq!(days_since_unix_epoch(1972, 3, 1), 790);
        }

        #[test]
        fn test_modern_date() {
            // January 1, 2000 (Y2K)
            // Between 1970 and 2000, there are 30 years.
            // Leap years: 1972, 76, 80, 84, 88, 92, 96 (7 leap years)
            // (23 non-leap * 365) + (7 leap * 366) = 10957
            assert_eq!(days_since_unix_epoch(2000, 1, 1), 10957);
        }
    }
}

/// Date and time representation.
///
/// # Integration
///
/// The type can be integrated with higher-level time libraries by using the
/// UTC UNIX timestamp in nanoseconds as exchange format. See:
/// - [`Self::from_utc_unix_timestamp_nanos`]
/// - [`Self::to_utc_unix_timestamp_nanos`]
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

    /// Creates a [`Time`] from the provided UNIX time stamp in nanoseconds
    /// (UTC).
    #[must_use]
    pub fn from_utc_unix_timestamp_nanos(nanos: i128) -> Self {
        let seconds = nanos.div_euclid(NANOS_PER_SECOND);
        let nanosecond = nanos.rem_euclid(NANOS_PER_SECOND) as u32;

        let mut days = seconds.div_euclid(SECONDS_PER_DAY as i128) as i64;
        let mut rem_secs = seconds.rem_euclid(SECONDS_PER_DAY as i128) as i64;

        let hour = (rem_secs / SECONDS_PER_HOUR) as u8;
        rem_secs %= SECONDS_PER_HOUR;

        let minute = (rem_secs / SECONDS_PER_MINUTE) as u8;
        let second = (rem_secs % SECONDS_PER_MINUTE) as u8;

        // --- Date reconstruction ---
        let mut year: i32 = 1970;

        loop {
            let year_days = if is_leap_year(year) { 366 } else { 365 };
            if days < year_days {
                break;
            }
            days -= year_days;
            year += 1;
        }

        let mut month: u8 = 1;
        loop {
            let dim = days_in_month(year, month) as i64;
            if days < dim {
                break;
            }
            days -= dim;
            month += 1;
        }

        let day = (days + 1) as u8;

        Self {
            year: year as u16,
            month,
            day,
            hour,
            minute,
            second,
            pad1: 0,
            nanosecond,
            time_zone: 0,
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

    /// Convert to signed nanoseconds since Unix epoch (UTC).
    ///
    /// Returns `None` if [`Self::is_valid`] returns `false`.
    #[must_use]
    pub fn to_utc_unix_timestamp_nanos(&self) -> Option<i128> {
        if !self.is_valid() {
            return None;
        }

        let days = days_since_unix_epoch(self.year as i32, self.month, self.day) as i128;

        let seconds = days * SECONDS_PER_DAY as i128
            + self.hour as i128 * SECONDS_PER_HOUR as i128
            + self.minute as i128 * SECONDS_PER_MINUTE as i128
            + self.second as i128;

        let tz_offset_seconds = match self.time_zone {
            Self::UNSPECIFIED_TIMEZONE => 0,
            minutes => minutes as i128 * 60,
        };

        let total_seconds = seconds - tz_offset_seconds;

        Some(total_seconds * NANOS_PER_SECOND + self.nanosecond as i128)
    }

    /// Returns the elapsed [`Duration`] since `other`, assuming that `self`
    /// happened later than `other`.
    ///
    /// Both dates need to be valid ([`Self::is_valid`]).
    #[must_use]
    pub fn elapsed_since(&self, other: &Self) -> Option<Duration> {
        let lhs = self.to_utc_unix_timestamp_nanos()?;
        let rhs = other.to_utc_unix_timestamp_nanos()?;

        if lhs < rhs {
            return None;
        }

        let delta = (lhs - rhs) as u128;
        Some(Duration::from_nanos(delta as u64))
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

impl PartialOrd for Time {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let lhs = self.to_utc_unix_timestamp_nanos()?;
        let rhs = other.to_utc_unix_timestamp_nanos()?;
        lhs.partial_cmp(&rhs)
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

    #[test]
    fn one_second_after_epoch() {
        let t = Time::from_utc_time(1970, 1, 1, 0, 0, 1, 0);
        let nanos = t.to_utc_unix_timestamp_nanos().unwrap();
        assert_eq!(nanos, NANOS_PER_SECOND);

        let t2 = Time::from_utc_unix_timestamp_nanos(NANOS_PER_SECOND);
        assert_eq!(t2.second, 1);
    }

    #[test]
    fn nanosecond_precision() {
        let t = Time::from_utc_time(1970, 1, 1, 0, 0, 0, 123_456_789);
        let nanos = t.to_utc_unix_timestamp_nanos().unwrap();
        assert_eq!(nanos, 123_456_789);

        let t2 = Time::from_utc_unix_timestamp_nanos(nanos);
        assert_eq!(t2.nanosecond, 123_456_789);
    }

    #[test]
    fn leap_day_2000() {
        // 2000-02-29 12:00:00 UTC
        let t = Time::from_utc_time(2000, 2, 29, 12, 0, 0, 0);
        let nanos = t.to_utc_unix_timestamp_nanos().unwrap();
        let t2 = Time::from_utc_unix_timestamp_nanos(nanos);

        assert_eq!(t2.year, 2000);
        assert_eq!(t2.month, 2);
        assert_eq!(t2.day, 29);
        assert_eq!(t2.hour, 12);
    }

    #[test]
    fn day_rollover() {
        // 1970-01-01 23:59:59 -> +1s
        let t = Time::from_utc_time(1970, 1, 1, 23, 59, 59, 0);
        let nanos = t.to_utc_unix_timestamp_nanos().unwrap() + NANOS_PER_SECOND;

        let t2 = Time::from_utc_unix_timestamp_nanos(nanos);
        assert_eq!(t2.year, 1970);
        assert_eq!(t2.month, 1);
        assert_eq!(t2.day, 2);
        assert_eq!(t2.hour, 0);
        assert_eq!(t2.minute, 0);
        assert_eq!(t2.second, 0);
    }

    #[test]
    fn round_trip_utc() {
        let t = Time::from_utc_time(2024, 10, 5, 17, 42, 9, 999_999_999);
        let nanos = t.to_utc_unix_timestamp_nanos().unwrap();
        let t2 = Time::from_utc_unix_timestamp_nanos(nanos);

        assert_eq!(t.year, t2.year);
        assert_eq!(t.month, t2.month);
        assert_eq!(t.day, t2.day);
        assert_eq!(t.hour, t2.hour);
        assert_eq!(t.minute, t2.minute);
        assert_eq!(t.second, t2.second);
        assert_eq!(t.nanosecond, t2.nanosecond);
        assert_eq!(t2.time_zone, 0);
    }

    #[test]
    fn timezone_offset_applied() {
        // 1970-01-01 01:00:00 with +60 minutes offset == epoch
        let t = Time {
            year: 1970,
            month: 1,
            day: 1,
            hour: 1,
            minute: 0,
            second: 0,
            pad1: 0,
            nanosecond: 0,
            time_zone: 60, // +1h
            daylight: Daylight::empty(),
            pad2: 0,
        };

        let nanos = t.to_utc_unix_timestamp_nanos().unwrap();
        assert_eq!(nanos, 0);
    }

    #[test]
    fn basic_elapsed() {
        let t1 = Time::from_utc_time(1970, 1, 1, 0, 0, 0, 0);
        let t2 = Time::from_utc_time(1970, 1, 1, 0, 0, 10, 500_000_000);

        let elapsed = t2.elapsed_since(&t1).unwrap();
        assert_eq!(elapsed, Duration::new(10, 500_000_000));

        // Zero elapsed
        let t3 = Time::from_utc_time(1970, 1, 1, 0, 0, 0, 0);
        let zero_elapsed = t3.elapsed_since(&t3).unwrap();
        assert_eq!(zero_elapsed, Duration::new(0, 0));

        let t4 = Time { day: 2, ..t1 };
        assert_eq!(
            t4.elapsed_since(&t3),
            Some(Duration::from_secs(SECONDS_PER_DAY as u64))
        );
    }

    #[test]
    fn self_before_other_returns_none() {
        let earlier = Time::from_utc_time(1970, 1, 1, 1, 0, 0, 0);
        let later = Time::from_utc_time(1970, 1, 1, 0, 0, 0, 0);

        let result = later.elapsed_since(&earlier);
        assert!(result.is_none());
    }

    #[test]
    fn invalid_time_returns_none() {
        let valid = Time::from_utc_time(1970, 1, 1, 0, 0, 0, 0);
        let mut invalid = Time::from_utc_time(1970, 1, 1, 0, 0, 0, 0);

        // make invalid (e.g., invalid month)
        invalid.month = 13;

        assert!(valid.elapsed_since(&invalid).is_none());
        assert!(invalid.elapsed_since(&valid).is_none());
    }

    #[test]
    fn leap_day_elapsed() {
        let t1 = Time::from_utc_time(2000, 2, 28, 23, 0, 0, 0);
        let t2 = Time::from_utc_time(2000, 2, 29, 1, 30, 0, 0);

        let elapsed = t2.elapsed_since(&t1).unwrap();
        let expected = Duration::from_secs(2 * 3600 + 30 * 60); // 2h30m
        assert_eq!(elapsed, expected);
    }
}
