// SPDX-License-Identifier: MIT OR Apache-2.0

//! Provides common helpers for the integration and conversion with
//! different time crates from the ecosystem.

use crate::runtime::TimeError;
use core::error::Error;
use core::fmt;
use core::fmt::{Display, Formatter};

/// An opaque error type indicating a UEFI [`Time`] could not be converted.
///
/// [`Time`]: super::Time
#[derive(Debug)]
pub struct ConversionError(pub(super) ConversionErrorInner);

impl Display for ConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Time conversion error: {}", self.0)
    }
}

impl Error for ConversionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // Don't expose the inner error, it is not useful to the user.
        None
    }
}

#[derive(Debug)]
pub(super) enum ConversionErrorInner {
    /// Invalid component.
    InvalidComponent,
    /// Invalid UEFI time: [`Time::is_valid`] reported an error.
    InvalidUefiTime(TimeError),
    /// A timezone was required for the conversion, but the UEFI time indicates
    /// [`Time::UNSPECIFIED_TIMEZONE`].
    ///
    /// [`Time::UNSPECIFIED_TIMEZONE`]: super::Time::UNSPECIFIED_TIMEZONE
    UnspecifiedTimezone,
    /// Errors raised in the [`time`] crate.
    #[cfg(feature = "time03")]
    TimeCrateError(time::Error),
    /// Errors raised in the [`jiff`] crate.
    #[cfg(feature = "jiff02")]
    JiffCrateError(jiff::Error),
}

impl Display for ConversionErrorInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidComponent => write!(f, "Invalid component"),
            Self::InvalidUefiTime(e) => write!(f, "Invalid UEFI time: {e}"),
            Self::UnspecifiedTimezone => write!(f, "Unspecified timezone"),
            #[cfg(feature = "time03")]
            Self::TimeCrateError(e) => write!(f, "Time crate error: {}", e),
            #[cfg(feature = "jiff02")]
            Self::JiffCrateError(e) => write!(f, "Jiff crate error: {}", e),
        }
    }
}

impl Error for ConversionErrorInner {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidComponent => None,
            Self::InvalidUefiTime(e) => Some(e),
            Self::UnspecifiedTimezone => None,
            #[cfg(feature = "time03")]
            Self::TimeCrateError(e) => Some(e),
            // None: Missing Error trait
            #[cfg(feature = "jiff02")]
            Self::JiffCrateError(_e) => None,
        }
    }
}

#[cfg(test)]
#[allow(unused)]
pub(super) mod test_helpers {
    use super::*;
    use crate::runtime::TimeParams;
    use uefi::runtime::Time;
    use uefi_raw::time::Daylight;

    pub fn sample_time(tz: i16) -> Time {
        let params = TimeParams {
            year: 2024,
            month: 3,
            day: 14,
            hour: 12,
            minute: 34,
            second: 56,
            nanosecond: 123_456_789,
            time_zone: Some(tz),
            daylight: Daylight::default(),
        };
        Time::new(params).unwrap()
    }

    fn invalid_date() -> Time {
        let mut t = sample_time(0);
        t.0.month = 2;
        t.0.day = 31;
        t
    }

    pub fn primitive_roundtrip<T>()
    where
        T: TryFrom<Time, Error = ConversionError> + TryInto<Time, Error = ConversionError>,
    {
        let t = sample_time(Time::UNSPECIFIED_TIMEZONE);

        let dt: T = t.try_into().unwrap();
        let back: Time = dt.try_into().unwrap();

        assert_eq!(back.0.year, t.0.year);
        assert_eq!(back.0.month, t.0.month);
        assert_eq!(back.0.day, t.0.day);
        assert_eq!(back.0.hour, t.0.hour);
        assert_eq!(back.0.minute, t.0.minute);
        assert_eq!(back.0.second, t.0.second);
        assert_eq!(back.0.nanosecond, t.0.nanosecond);
    }

    pub fn zoned_roundtrip<T>()
    where
        T: TryFrom<Time, Error = ConversionError> + TryInto<Time, Error = ConversionError>,
    {
        let t = sample_time(120);

        let dt: T = t.try_into().unwrap();
        let back: Time = dt.try_into().unwrap();

        assert_eq!(back.0.time_zone, 120);
        assert_eq!(back.0.hour, t.0.hour);
        assert_eq!(back.0.minute, t.0.minute);
    }

    pub fn negative_offset_roundtrip<T>()
    where
        T: TryFrom<Time, Error = ConversionError> + TryInto<Time, Error = ConversionError>,
    {
        let t = sample_time(-330);

        let dt: T = t.try_into().unwrap();
        let back: Time = dt.try_into().unwrap();

        assert_eq!(back.0.time_zone, -330);
    }

    pub fn preserves_nanoseconds<T>()
    where
        T: TryFrom<Time, Error = ConversionError> + TryInto<Time, Error = ConversionError>,
    {
        let t = sample_time(60);

        let dt: T = t.try_into().unwrap();
        let back: Time = dt.try_into().unwrap();

        assert_eq!(back.0.nanosecond, 123_456_789);
    }

    pub fn unspecified_timezone_fails<T>()
    where
        T: TryFrom<Time, Error = ConversionError>,
    {
        let t = sample_time(Time::UNSPECIFIED_TIMEZONE);
        let result: Result<T, _> = t.try_into();
        assert!(result.is_err());
    }

    pub fn invalid_calendar_date_fails<T>()
    where
        T: TryFrom<Time, Error = ConversionError>,
    {
        let result: Result<T, _> = invalid_date().try_into();
        assert!(result.is_err());
    }
}
