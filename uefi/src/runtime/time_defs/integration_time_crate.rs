// SPDX-License-Identifier: MIT OR Apache-2.0

//! Integration of the UEFI [`Time`] type with the [`time`] crate.

use super::Time;
use super::integration_common::{ConversionError, ConversionErrorInner};
use crate::runtime::TimeParams;
use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset};

impl TryFrom<Time> for PrimitiveDateTime {
    type Error = ConversionError;

    fn try_from(value: Time) -> Result<Self, Self::Error> {
        if let Err(e) = value.is_valid() {
            return Err(ConversionError(ConversionErrorInner::InvalidUefiTime(e)));
        }

        // Emulated try {} block to keep the `?` error propagation scoped
        // (we have a different error type here)
        let datetime: Result<Self, time::error::ComponentRange> = (|| {
            let month = time::Month::try_from(value.0.month)?;
            let date = time::Date::from_calendar_date(value.0.year as i32, month, value.0.day)?;
            let time = time::Time::from_hms_nano(
                value.0.hour,
                value.0.minute,
                value.0.second,
                value.0.nanosecond,
            )?;
            Ok(Self::new(date, time))
        })();

        datetime
            .map_err(time::Error::ComponentRange)
            .map_err(|e| ConversionError(ConversionErrorInner::TimeCrateError(e)))
    }
}

impl TryFrom<Time> for OffsetDateTime {
    type Error = ConversionError;

    fn try_from(value: Time) -> Result<Self, Self::Error> {
        if let Err(e) = value.is_valid() {
            return Err(ConversionError(ConversionErrorInner::InvalidUefiTime(e)));
        }

        let primitive_date_time: PrimitiveDateTime = value.try_into()?;

        if value.0.time_zone == Time::UNSPECIFIED_TIMEZONE {
            return Err(ConversionError(ConversionErrorInner::UnspecifiedTimezone));
        }

        let h = (value.0.time_zone / 60) as i8;
        let m = (value.0.time_zone % 60) as i8;

        let offset = UtcOffset::from_hms(h, m, 0)
            .map_err(time::Error::ComponentRange)
            .map_err(|e| ConversionError(ConversionErrorInner::TimeCrateError(e)))?;
        Ok(primitive_date_time.assume_offset(offset))
    }
}

impl TryFrom<PrimitiveDateTime> for Time {
    type Error = ConversionError;

    fn try_from(value: PrimitiveDateTime) -> Result<Self, Self::Error> {
        let params = TimeParams {
            year: u16::try_from(value.year())
                .map_err(|_e| ConversionError(ConversionErrorInner::InvalidComponent))?,
            month: u8::from(value.month()),
            day: value.day(),
            hour: value.hour(),
            minute: value.minute(),
            second: value.second(),
            nanosecond: value.nanosecond(),
            time_zone: None,
            daylight: Default::default(),
        };
        Self::new(params).map_err(|e| ConversionError(ConversionErrorInner::InvalidUefiTime(e)))
    }
}

impl TryFrom<OffsetDateTime> for Time {
    type Error = ConversionError;

    fn try_from(value: OffsetDateTime) -> Result<Self, Self::Error> {
        let timezone_offset_minutes = value.offset().whole_seconds() / 60;
        if value.offset().whole_seconds() % 60 != 0 {
            return Err(ConversionError(ConversionErrorInner::InvalidComponent));
        }

        let params = TimeParams {
            year: u16::try_from(value.year())
                .map_err(|_e| ConversionError(ConversionErrorInner::InvalidComponent))?,
            month: u8::from(value.month()),
            day: value.day(),
            hour: value.hour(),
            minute: value.minute(),
            second: value.second(),
            nanosecond: value.nanosecond(),
            time_zone: Some(
                i16::try_from(timezone_offset_minutes)
                    .map_err(|_e| ConversionError(ConversionErrorInner::InvalidComponent))?,
            ),
            daylight: Default::default(),
        };

        Self::new(params).map_err(|e| ConversionError(ConversionErrorInner::InvalidUefiTime(e)))
    }
}

#[cfg(test)]
mod tests {
    use super::super::integration_common::test_helpers;
    use super::*;
    use time::{OffsetDateTime, PrimitiveDateTime};

    #[test]
    fn primitive_roundtrip_basic() {
        test_helpers::primitive_roundtrip::<PrimitiveDateTime>();
    }

    #[test]
    fn zoned_roundtrip_positive_offset() {
        test_helpers::zoned_roundtrip::<OffsetDateTime>();
    }

    #[test]
    fn zoned_roundtrip_negative_offset() {
        test_helpers::negative_offset_roundtrip::<OffsetDateTime>();
    }

    #[test]
    fn nanoseconds_preserved() {
        test_helpers::preserves_nanoseconds::<OffsetDateTime>();
    }

    #[test]
    fn unspecified_timezone_is_rejected() {
        test_helpers::unspecified_timezone_fails::<OffsetDateTime>();
    }

    #[test]
    fn invalid_date_is_rejected() {
        test_helpers::invalid_calendar_date_fails::<PrimitiveDateTime>();
    }

    // important real-world offset edge case
    #[test]
    fn half_hour_timezone_roundtrip() {
        let t = test_helpers::sample_time(90); // +01:30

        let dt: OffsetDateTime = t.try_into().unwrap();
        let back: Time = dt.try_into().unwrap();

        assert_eq!(back.0.time_zone, 90);
    }

    #[test]
    fn negative_half_hour_timezone_roundtrip() {
        let t = test_helpers::sample_time(-330); // -05:30

        let dt: OffsetDateTime = t.try_into().unwrap();
        let back: Time = dt.try_into().unwrap();

        assert_eq!(back.0.time_zone, -330);
    }
}
