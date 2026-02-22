// SPDX-License-Identifier: MIT OR Apache-2.0

//! Date and time types.

#[cfg(feature = "time")]
pub use time_crate_integration::*;

use bitflags::bitflags;
use core::fmt::{self, Display, Formatter};

/// Date and time representation.
///
/// # Integration with `time` crate
///
/// This type has a close integration with the [`time` crate][time] for correct
/// and convenient time handling. You can use:
///
/// - `Time::to_offset_date_time`
/// - `Time::to_offset_date_time_with_default_timezone`
/// - [`TryFrom`] from `OffsetDateTime` to [`Time`]
///
/// [time]: https://crates.io/crates/time
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
        /// Daylight information not available or not applicable to the time
        /// zone.
        const NONE = 0;

        /// Time is affected by daylight savings time.
        const ADJUST_DAYLIGHT = 0x01;

        /// Time has been adjusted for daylight savings time.
        const IN_DAYLIGHT = 0x02;
    }
}

#[cfg(feature = "time")]
mod time_crate_integration {
    use super::*;
    use core::error;
    use time::{OffsetDateTime, UtcOffset};

    /// The time is invalid as it doesn't fulfill the requirements of
    /// [`Time::is_valid`].
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct InvalidTimeError(Time);

    impl Display for InvalidTimeError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "Invalid EFI time (not valid): {}", self.0)
        }
    }

    impl error::Error for InvalidTimeError {}

    /// Errors that can happen when converting a [`Time`] to a [`OffsetDateTime`].
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum ToOffsetDateTimeError {
        /// See [`InvalidTimeError`].
        Invalid(InvalidTimeError),
        /// The timezone is a valid EFI value but [`Time::UNSPECIFIED_TIMEZONE`]
        /// means `local` which cannot be determined in a `no_std` context.
        UnspecifiedTimezone,
        /// The corresponding component has an invalid range (e.g. February 30th).
        ComponentRange(time::error::ComponentRange),
    }

    impl Display for ToOffsetDateTimeError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                Self::Invalid(e) => {
                    write!(f, "Cannot convert to OffsetDateTime: {}", e)
                }
                Self::UnspecifiedTimezone => write!(
                    f,
                    "Cannot convert to OffsetDateTime: the timezone (local) can't be determined in a no_std context"
                ),
                Self::ComponentRange(e) => {
                    write!(f, "Cannot convert to OffsetDateTime: {}", e)
                }
            }
        }
    }

    impl error::Error for ToOffsetDateTimeError {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            match self {
                Self::Invalid(e) => Some(e),
                Self::UnspecifiedTimezone => None,
                Self::ComponentRange(e) => Some(e),
            }
        }
    }

    impl Time {
        fn _to_offset_date_time(
            &self,
            local_timezone: Option<UtcOffset>,
        ) -> Result<OffsetDateTime, ToOffsetDateTimeError> {
            if !self.is_valid() {
                return Err(ToOffsetDateTimeError::Invalid(InvalidTimeError(*self)));
            }

            // Special handling for `UNSPECIFIED_TIMEZONE` (local):
            // - specific time zone => all good
            // - `UNSPECIFIED_TIMEZONE` and no default time zone => return Err
            // - else: use the default time zone
            let offset_hms: (i8, i8, i8) = {
                if self.time_zone == Self::UNSPECIFIED_TIMEZONE {
                    if let Some(local_timezone) = local_timezone {
                        local_timezone.as_hms()
                    } else {
                        return Err(ToOffsetDateTimeError::UnspecifiedTimezone);
                    }
                } else {
                    let h = self.time_zone / 60;
                    let m = self.time_zone % 60;

                    (h as i8, m as i8, 0)
                }
            };

            // Emulated try {} block to keep the `?` error propagation scoped
            // (we have a different error type here)
            let datetime: Result<OffsetDateTime, time::error::ComponentRange> = (|| {
                let month = time::Month::try_from(self.month)?;
                let date = time::Date::from_calendar_date(self.year as i32, month, self.day)?;
                let time = time::Time::from_hms_nano(
                    self.hour,
                    self.minute,
                    self.second,
                    self.nanosecond,
                )?;
                let offset = UtcOffset::from_hms(offset_hms.0, offset_hms.1, offset_hms.2)?;
                Ok(OffsetDateTime::new_in_offset(date, time, offset))
            })();

            datetime.map_err(ToOffsetDateTimeError::ComponentRange)
        }

        /// Converts this [`Time`] to a [`OffsetDateTime`] using the UTC offset
        /// stored in `self.time_zone`.
        ///
        /// # Returns
        ///
        /// - `Ok(OffsetDateTime)` if the time is valid and the timezone is fully
        ///   specified (not `UNSPECIFIED_TIMEZONE`).
        /// - `Err(ToOffsetDateTimeError::Invalid(_))` if any of the time fields are
        ///   out of valid EFI ranges (e.g., year < 1900, month > 12).
        /// - `Err(ToOffsetDateTimeError::UnspecifiedTimezone)` if the timezone is
        ///   `UNSPECIFIED_TIMEZONE`, since no default local timezone was provided.
        /// - `Err(ToOffsetDateTimeError::ComponentRange(_))` if the combination of
        ///   fields results in an invalid calendar date (e.g., February 30th).
        ///
        /// # Examples
        ///
        /// ```
        /// # use uefi_raw::time::{Time, ToOffsetDateTimeError};
        /// # use time::OffsetDateTime;
        /// let t = Time {
        ///     year: 2024,
        ///     month: 3,
        ///     day: 15,
        ///     hour: 12,
        ///     minute: 0,
        ///     second: 0,
        ///     pad1: 0,
        ///     nanosecond: 0,
        ///     time_zone: 0, // UTC
        ///     daylight: Default::default(),
        ///     pad2: 0,
        /// };
        ///
        /// let odt: OffsetDateTime = t.to_offset_date_time().unwrap();
        /// assert_eq!(odt.offset().whole_seconds(), 0);
        /// ```
        pub fn to_offset_date_time(&self) -> Result<OffsetDateTime, ToOffsetDateTimeError> {
            self._to_offset_date_time(None)
        }

        /// Converts this [`Time`] to a [`OffsetDateTime`] using a provided default
        /// timezone when `self.time_zone` is [`Self::UNSPECIFIED_TIMEZONE`].
        ///
        /// If the stored `time_zone` field is `UNSPECIFIED_TIMEZONE`, this method
        /// uses the provided `local` [`UtcOffset`] as the offset. Otherwise, the
        /// stored `time_zone` in minutes is used.
        ///
        /// # Arguments
        ///
        /// - `local`: The default [`UtcOffset`] to use if `time_zone` is
        ///   [`Self::UNSPECIFIED_TIMEZONE`].
        ///
        /// # Returns
        ///
        /// - `Ok(OffsetDateTime)` if the time is valid and the offset can be
        ///   applied successfully.
        /// - `Err(ToOffsetDateTimeError::Invalid(_))` if any of the time fields are
        ///   out of valid EFI ranges.
        /// - `Err(ToOffsetDateTimeError::ComponentRange(_))` if the combination of
        ///   fields results in an invalid calendar date.
        ///
        /// # Panics
        ///
        /// This function does **not panic** under normal usage. It is impossible
        /// for `UnspecifiedTimezone` to be returned because the caller provides
        /// a valid default offset.
        ///
        /// # Examples
        ///
        /// ```
        /// # use uefi_raw::time::{Time, ToOffsetDateTimeError};
        /// # use time::{OffsetDateTime, UtcOffset};
        /// let mut t = Time {
        ///     year: 2024,
        ///     month: 3,
        ///     day: 15,
        ///     hour: 12,
        ///     minute: 0,
        ///     second: 0,
        ///     pad1: 0,
        ///     nanosecond: 0,
        ///     time_zone: Time::UNSPECIFIED_TIMEZONE,
        ///     daylight: Default::default(),
        ///     pad2: 0,
        /// };
        ///
        /// let default_offset = UtcOffset::from_hms(1, 30, 0).unwrap();
        /// let odt: OffsetDateTime = t.to_offset_date_time_with_default_timezone(default_offset).unwrap();
        /// assert_eq!(odt.offset(), default_offset);
        /// ```
        pub fn to_offset_date_time_with_default_timezone(
            &self,
            local: UtcOffset,
        ) -> Result<OffsetDateTime, ToOffsetDateTimeError> {
            match self._to_offset_date_time(Some(local)) {
                Err(ToOffsetDateTimeError::UnspecifiedTimezone) => unreachable!(),
                other => other,
            }
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum FromOffsetDateTimeError {
        /// The year is not representable as `u16`.
        InvalidYear(i32),
        /// Time zone offsets with seconds are not representable.
        OffsetWithSeconds(i8),
        InvalidTime(InvalidTimeError),
    }

    impl Display for FromOffsetDateTimeError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                Self::InvalidYear(y) => {
                    write!(
                        f,
                        "Cannot to convert OffsetDateTime to EFI Time: invalid year {y}"
                    )
                }
                Self::OffsetWithSeconds(s) => {
                    write!(
                        f,
                        "Cannot to convert OffsetDateTime to EFI Time: time zone offset has seconds ({s}) which is unsupported"
                    )
                }
                Self::InvalidTime(t) => {
                    write!(f, "The time is invalid: {t}")
                }
            }
        }
    }

    impl error::Error for FromOffsetDateTimeError {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            match self {
                Self::InvalidYear(_) => None,
                Self::OffsetWithSeconds(_) => None,
                Self::InvalidTime(e) => Some(e),
            }
        }
    }

    impl TryFrom<OffsetDateTime> for Time {
        type Error = FromOffsetDateTimeError;

        fn try_from(value: OffsetDateTime) -> Result<Self, Self::Error> {
            let this = Self {
                year: u16::try_from(value.date().year())
                    .map_err(|_| Self::Error::InvalidYear(value.date().year()))?,
                // No checks needed: underlying type has repr `u8`
                month: value.date().month() as u8,
                day: value.date().day(),
                hour: value.time().hour(),
                minute: value.time().minute(),
                second: value.time().second(),
                pad1: 0,
                nanosecond: value.time().nanosecond(),
                time_zone: {
                    let (h, m, s) = value.offset().as_hms();
                    if s != 0 {
                        return Err(Self::Error::OffsetWithSeconds(s));
                    }
                    (h as i16 * 60) + (m as i16)
                },
                daylight: Daylight::NONE,
                pad2: 0,
            };
            if this.is_valid() {
                Ok(this)
            } else {
                Err(Self::Error::InvalidTime(InvalidTimeError(this)))
            }
        }
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

#[cfg(all(test, feature = "time"))]
mod to_offset_date_time_tests {
    use super::*;
    use time::UtcOffset;

    fn create_valid_efi_time() -> Time {
        Time {
            year: 2024,
            month: 2,
            day: 20,
            hour: 10,
            minute: 0,
            second: 0,
            nanosecond: 0,
            time_zone: 60, // UTC+1
            daylight: Daylight::NONE,
            ..Time::default()
        }
    }

    #[test]
    fn test_basic_conversion() {
        let t = create_valid_efi_time();
        let odt = t.to_offset_date_time().unwrap();

        assert_eq!(odt.year(), 2024);
        assert_eq!(odt.month() as u8, 2);
        assert_eq!(odt.day(), 20);
        assert_eq!(odt.hour(), 10);
        assert_eq!(odt.offset().whole_minutes(), 60);
    }

    #[test]
    fn test_unspecified_timezone_error() {
        let mut t = create_valid_efi_time();
        t.time_zone = Time::UNSPECIFIED_TIMEZONE;

        let result = t.to_offset_date_time();
        assert!(matches!(
            result,
            Err(ToOffsetDateTimeError::UnspecifiedTimezone)
        ));
    }

    #[test]
    fn test_unspecified_timezone_with_default() {
        let mut t = create_valid_efi_time();
        t.time_zone = Time::UNSPECIFIED_TIMEZONE;

        let default_offset = UtcOffset::from_hms(-5, 0, 0).unwrap();
        let odt = t
            .to_offset_date_time_with_default_timezone(default_offset)
            .unwrap();

        assert_eq!(odt.offset(), default_offset);
        assert_eq!(odt.hour(), 10);
    }

    #[test]
    fn test_invalid_efi_fields() {
        let mut t = create_valid_efi_time();
        t.month = 13; // Invalid EFI month

        let result = t.to_offset_date_time();
        assert!(matches!(result, Err(ToOffsetDateTimeError::Invalid(_))));
    }

    #[test]
    fn test_calendar_component_range_error() {
        let mut t = create_valid_efi_time();
        t.year = 2023;
        t.month = 2;
        t.day = 29; // 2023 is not a leap year

        let result = t.to_offset_date_time();
        assert!(matches!(
            result,
            Err(ToOffsetDateTimeError::ComponentRange(_))
        ));
    }

    #[test]
    fn test_leap_year_valid() {
        let mut t = create_valid_efi_time();
        t.year = 2024;
        t.month = 2;
        t.day = 29; // 2024 is a leap year

        assert!(t.to_offset_date_time().is_ok());
    }

    #[test]
    fn test_max_timezone_offset() {
        let mut t = create_valid_efi_time();
        t.time_zone = 1440; // Max valid EFI offset (+24h)

        let odt = t.to_offset_date_time().unwrap();
        assert_eq!(odt.offset().whole_hours(), 24);
    }

    #[test]
    fn test_min_timezone_offset() {
        let mut t = create_valid_efi_time();
        t.time_zone = -1440; // -24:00

        let odt = t.to_offset_date_time().unwrap();
        assert_eq!(odt.offset().whole_hours(), -24);
    }

    #[test]
    fn test_nanosecond_boundaries() {
        let mut t = create_valid_efi_time();
        t.nanosecond = 0;
        assert!(t.to_offset_date_time().is_ok());

        t.nanosecond = 999_999_999;
        assert!(t.to_offset_date_time().is_ok());
    }
}

#[cfg(all(test, feature = "time"))]
mod try_from_offset_datetime_tests {
    use super::*;
    use time::{Date, OffsetDateTime, UtcOffset};

    fn make_odt(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
        offset_minutes: i16,
    ) -> OffsetDateTime {
        let date =
            Date::from_calendar_date(year, time::Month::try_from(month).unwrap(), day).unwrap();
        let time = time::Time::from_hms_nano(hour, minute, second, nanosecond).unwrap();
        let offset =
            UtcOffset::from_hms((offset_minutes / 60) as i8, (offset_minutes % 60) as i8, 0)
                .unwrap();
        OffsetDateTime::new_in_offset(date, time, offset)
    }

    #[test]
    fn test_happy_path() {
        let odt = make_odt(2024, 3, 15, 12, 30, 45, 123_456_789, 120);
        let t = Time::try_from(odt).unwrap();

        assert_eq!(t.year, 2024);
        assert_eq!(t.month, 3);
        assert_eq!(t.day, 15);
        assert_eq!(t.hour, 12);
        assert_eq!(t.minute, 30);
        assert_eq!(t.second, 45);
        assert_eq!(t.nanosecond, 123_456_789);
        assert_eq!(t.time_zone, 120);
    }

    #[test]
    fn test_negative_timezone() {
        let odt = make_odt(2024, 1, 1, 0, 0, 0, 0, -330);
        let t = Time::try_from(odt).unwrap();
        assert_eq!(t.time_zone, -330);
    }

    #[test]
    fn test_offset_with_seconds_fails() {
        let date = Date::from_calendar_date(2024, time::Month::January, 1).unwrap();
        let time = time::Time::from_hms_nano(0, 0, 0, 0).unwrap();
        let offset = UtcOffset::from_hms(1, 0, 30).unwrap();
        let odt = OffsetDateTime::new_in_offset(date, time, offset);

        let result = Time::try_from(odt);
        assert!(matches!(
            result,
            Err(FromOffsetDateTimeError::OffsetWithSeconds(30))
        ));
    }

    #[test]
    fn test_invalid_year_too_low() {
        let odt = make_odt(1800, 1, 1, 0, 0, 0, 0, 0);
        let result = Time::try_from(odt);
        assert!(matches!(
            result,
            Err(FromOffsetDateTimeError::InvalidTime(_))
        ));
    }

    #[test]
    fn test_boundary_years() {
        let min_valid = make_odt(1900, 1, 1, 0, 0, 0, 0, 0);
        assert!(Time::try_from(min_valid).is_ok());

        let max_valid = make_odt(9999, 12, 31, 23, 59, 59, 999_999_999, 0);
        assert!(Time::try_from(max_valid).is_ok());
    }

    #[test]
    fn test_round_trip_conversion() {
        let odt = make_odt(2024, 5, 6, 7, 8, 9, 987_654_321, 180);
        let t = Time::try_from(odt).unwrap();
        let odt2 = t
            .to_offset_date_time_with_default_timezone(UtcOffset::from_hms(3, 0, 0).unwrap())
            .unwrap();

        assert_eq!(odt.year(), odt2.year());
        assert_eq!(odt.month(), odt2.month());
        assert_eq!(odt.day(), odt2.day());
        assert_eq!(odt.hour(), odt2.hour());
        assert_eq!(odt.minute(), odt2.minute());
        assert_eq!(odt.second(), odt2.second());
        assert_eq!(odt.nanosecond(), odt2.nanosecond());
        assert_eq!(odt.offset().whole_minutes(), odt2.offset().whole_minutes());
    }
}
