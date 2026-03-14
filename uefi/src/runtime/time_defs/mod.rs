// SPDX-License-Identifier: MIT OR Apache-2.0

//! Module for UEFI time-related types and definitions and convenience and
//! abstractions build around these.

use core::fmt;
use core::fmt::{Debug, Display, Formatter};
use uefi_raw::time::Daylight;

#[allow(unused)]
mod integration_common;

/// Date and time representation.
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Time(uefi_raw::time::Time);

/// Input parameters for [`Time::new`].
#[derive(Copy, Clone, Debug)]
pub struct TimeParams {
    /// Year in the range `1900..=9999`.
    pub year: u16,

    /// Month in the range `1..=12`.
    pub month: u8,

    /// Day in the range `1..=31`.
    pub day: u8,

    /// Hour in the range `0.=23`.
    pub hour: u8,

    /// Minute in the range `0..=59`.
    pub minute: u8,

    /// Second in the range `0..=59`.
    pub second: u8,

    /// Fraction of a second represented as nanoseconds in the range
    /// `0..=999_999_999`.
    pub nanosecond: u32,

    /// Offset in minutes from UTC in the range `-1440..=1440`, or
    /// local time if `None`.
    pub time_zone: Option<i16>,

    /// Daylight savings time information.
    pub daylight: Daylight,
}

/// Error returned by [`Time`] methods. A bool value of `true` means
/// the specified field is outside its valid range.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TimeError {
    pub year: bool,
    pub month: bool,
    pub day: bool,
    pub hour: bool,
    pub minute: bool,
    pub second: bool,
    pub nanosecond: bool,
    pub timezone: bool,
    pub daylight: bool,
}

impl core::error::Error for TimeError {}

impl Display for TimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.year {
            writeln!(f, "year not within `1900..=9999`")?;
        }
        if self.month {
            writeln!(f, "month not within `1..=12")?;
        }
        if self.day {
            writeln!(f, "day not within `1..=31`")?;
        }
        if self.hour {
            writeln!(f, "hour not within `0..=23`")?;
        }
        if self.minute {
            writeln!(f, "minute not within `0..=59`")?;
        }
        if self.second {
            writeln!(f, "second not within `0..=59`")?;
        }
        if self.nanosecond {
            writeln!(f, "nanosecond not within `0..=999_999_999`")?;
        }
        if self.timezone {
            writeln!(
                f,
                "time_zone not `Time::UNSPECIFIED_TIMEZONE` nor within `-1440..=1440`"
            )?;
        }
        if self.daylight {
            writeln!(f, "unknown bits set for daylight")?;
        }
        Ok(())
    }
}

impl Time {
    /// Unspecified Timezone/local time.
    const UNSPECIFIED_TIMEZONE: i16 = uefi_raw::time::Time::UNSPECIFIED_TIMEZONE;

    /// Create a `Time` value. If a field is not in the valid range,
    /// [`TimeError`] is returned.
    pub fn new(params: TimeParams) -> core::result::Result<Self, TimeError> {
        let time = Self(uefi_raw::time::Time {
            year: params.year,
            month: params.month,
            day: params.day,
            hour: params.hour,
            minute: params.minute,
            second: params.second,
            pad1: 0,
            nanosecond: params.nanosecond,
            time_zone: params.time_zone.unwrap_or(Self::UNSPECIFIED_TIMEZONE),
            daylight: params.daylight,
            pad2: 0,
        });

        time.is_valid().map(|_| time)
    }

    /// Create an invalid `Time` with all fields set to zero. This can
    /// be used with [`FileInfo`] to indicate a field should not be
    /// updated when calling [`File::set_info`].
    ///
    /// [`FileInfo`]: uefi::proto::media::file::FileInfo
    /// [`File::set_info`]: uefi::proto::media::file::File::set_info
    #[must_use]
    pub const fn invalid() -> Self {
        Self(uefi_raw::time::Time::invalid())
    }

    /// `Ok()` if all fields are within valid ranges, `Err(TimeError)` otherwise.
    pub fn is_valid(&self) -> core::result::Result<(), TimeError> {
        let mut err = TimeError::default();
        if !(1900..=9999).contains(&self.year()) {
            err.year = true;
        }
        if !(1..=12).contains(&self.month()) {
            err.month = true;
        }
        if !(1..=31).contains(&self.day()) {
            err.day = true;
        }
        if self.hour() > 23 {
            err.hour = true;
        }
        if self.minute() > 59 {
            err.minute = true;
        }
        if self.second() > 59 {
            err.second = true;
        }
        if self.nanosecond() > 999_999_999 {
            err.nanosecond = true;
        }
        if self.time_zone().is_some() && !((-1440..=1440).contains(&self.time_zone().unwrap())) {
            err.timezone = true;
        }
        // All fields are false, i.e., within their valid range.
        if err == TimeError::default() {
            Ok(())
        } else {
            Err(err)
        }
    }

    /// Query the year.
    #[must_use]
    pub const fn year(&self) -> u16 {
        self.0.year
    }

    /// Query the month.
    #[must_use]
    pub const fn month(&self) -> u8 {
        self.0.month
    }

    /// Query the day.
    #[must_use]
    pub const fn day(&self) -> u8 {
        self.0.day
    }

    /// Query the hour.
    #[must_use]
    pub const fn hour(&self) -> u8 {
        self.0.hour
    }

    /// Query the minute.
    #[must_use]
    pub const fn minute(&self) -> u8 {
        self.0.minute
    }

    /// Query the second.
    #[must_use]
    pub const fn second(&self) -> u8 {
        self.0.second
    }

    /// Query the nanosecond.
    #[must_use]
    pub const fn nanosecond(&self) -> u32 {
        self.0.nanosecond
    }

    /// Query the time offset in minutes from UTC, or None if using local time.
    #[must_use]
    pub const fn time_zone(&self) -> Option<i16> {
        if self.0.time_zone == Self::UNSPECIFIED_TIMEZONE {
            None
        } else {
            Some(self.0.time_zone)
        }
    }

    /// Query the daylight savings time information.
    #[must_use]
    pub const fn daylight(&self) -> Daylight {
        self.0.daylight
    }
}

impl Debug for Time {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02} ",
            self.0.year, self.0.month, self.0.day
        )?;
        write!(
            f,
            "{:02}:{:02}:{:02}.{:09}",
            self.0.hour, self.0.minute, self.0.second, self.0.nanosecond
        )?;
        if self.0.time_zone == Self::UNSPECIFIED_TIMEZONE {
            write!(f, ", Timezone=local")?;
        } else {
            write!(f, ", Timezone={}", self.0.time_zone)?;
        }
        write!(f, ", Daylight={:?}", self.0.daylight)
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Error returned from failing to convert a byte slice into a [`Time`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeByteConversionError {
    /// One or more fields of the converted [`Time`] is invalid.
    InvalidFields(TimeError),
    /// The byte slice is not large enough to hold a [`Time`].
    InvalidSize,
}

impl Display for TimeByteConversionError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::InvalidFields(error) => write!(f, "{error}"),
            Self::InvalidSize => write!(
                f,
                "the byte slice is not large enough to hold a Time struct"
            ),
        }
    }
}

impl TryFrom<&[u8]> for Time {
    type Error = TimeByteConversionError;

    fn try_from(bytes: &[u8]) -> core::result::Result<Self, Self::Error> {
        if size_of::<Self>() <= bytes.len() {
            let year = u16::from_le_bytes(bytes[0..2].try_into().unwrap());
            let month = bytes[2];
            let day = bytes[3];
            let hour = bytes[4];
            let minute = bytes[5];
            let second = bytes[6];
            let nanosecond = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
            let time_zone = match i16::from_le_bytes(bytes[12..14].try_into().unwrap()) {
                Self::UNSPECIFIED_TIMEZONE => None,
                num => Some(num),
            };
            let daylight = Daylight::from_bits(bytes[14]).ok_or_else(|| {
                TimeByteConversionError::InvalidFields(TimeError {
                    daylight: true,
                    ..Default::default()
                })
            })?;

            let time_params = TimeParams {
                year,
                month,
                day,
                hour,
                minute,
                second,
                nanosecond,
                time_zone,
                daylight,
            };

            Self::new(time_params).map_err(TimeByteConversionError::InvalidFields)
        } else {
            Err(TimeByteConversionError::InvalidSize)
        }
    }
}
