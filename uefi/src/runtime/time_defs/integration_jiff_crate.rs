// SPDX-License-Identifier: MIT OR Apache-2.0

//! Integration of the UEFI [`Time`] type with the [`jiff`] crate.

use super::Time;
use super::integration_common::{ConversionError, ConversionErrorInner};
use jiff::Zoned;
use jiff::civil::DateTime;
use jiff::tz::{Offset, TimeZone};
use uefi::runtime::TimeParams;

// Timezone unaware
impl TryFrom<Time> for DateTime {
    type Error = ConversionError;

    fn try_from(value: Time) -> Result<Self, Self::Error> {
        if let Err(e) = value.is_valid() {
            return Err(ConversionError(ConversionErrorInner::InvalidUefiTime(e)));
        }

        let datetime = Self::new(
            // Cannot fail as the value is valid and in range (we checked that).
            i16::try_from(value.0.year).unwrap(),
            // Cannot fail as the value is valid and in range (we checked that).
            i8::try_from(value.0.month).unwrap(),
            // Cannot fail as the value is valid and in range (we checked that).
            i8::try_from(value.0.day).unwrap(),
            // Cannot fail as the value is valid and in range (we checked that).
            i8::try_from(value.0.hour).unwrap(),
            // Cannot fail as the value is valid and in range (we checked that).
            i8::try_from(value.0.minute).unwrap(),
            // Cannot fail as the value is valid and in range (we checked that).
            i8::try_from(value.0.second).unwrap(),
            // Cannot fail as the value is valid and in range (we checked that).
            i32::try_from(value.0.nanosecond).unwrap(),
        )
        .map_err(|e| ConversionError(ConversionErrorInner::JiffCrateError(e)))?;

        Ok(datetime)
    }
}

// Timezone aware
impl TryFrom<Time> for Zoned {
    type Error = ConversionError;

    fn try_from(value: Time) -> Result<Self, Self::Error> {
        if let Err(e) = value.is_valid() {
            return Err(ConversionError(ConversionErrorInner::InvalidUefiTime(e)));
        }

        if value.0.time_zone == Time::UNSPECIFIED_TIMEZONE {
            return Err(ConversionError(ConversionErrorInner::UnspecifiedTimezone));
        }

        let datetime = DateTime::try_from(value)?;
        let seconds = value.0.time_zone as i32 * 60 /* seconds per minute */;
        let offset = Offset::from_seconds(seconds)
            .map_err(|e| ConversionError(ConversionErrorInner::JiffCrateError(e)))?;
        let timezone = TimeZone::fixed(offset);
        let zoned = datetime
            .to_zoned(timezone)
            .map_err(|e| ConversionError(ConversionErrorInner::JiffCrateError(e)))?;
        Ok(zoned)
    }
}

impl TryFrom<DateTime> for Time {
    type Error = ConversionError;

    fn try_from(value: DateTime) -> Result<Self, Self::Error> {
        let params = TimeParams {
            year: u16::try_from(value.year())
                .map_err(|_e| ConversionError(ConversionErrorInner::InvalidComponent))?,
            month: u8::try_from(value.month())
                .map_err(|_e| ConversionError(ConversionErrorInner::InvalidComponent))?,
            day: u8::try_from(value.day())
                .map_err(|_e| ConversionError(ConversionErrorInner::InvalidComponent))?,
            hour: u8::try_from(value.hour())
                .map_err(|_e| ConversionError(ConversionErrorInner::InvalidComponent))?,
            minute: u8::try_from(value.minute())
                .map_err(|_e| ConversionError(ConversionErrorInner::InvalidComponent))?,
            second: u8::try_from(value.second())
                .map_err(|_e| ConversionError(ConversionErrorInner::InvalidComponent))?,
            nanosecond: u32::try_from(value.subsec_nanosecond())
                .map_err(|_e| ConversionError(ConversionErrorInner::InvalidComponent))?,
            time_zone: None,
            daylight: Default::default(),
        };
        Self::new(params).map_err(|e| ConversionError(ConversionErrorInner::InvalidUefiTime(e)))
    }
}

impl TryFrom<Zoned> for Time {
    type Error = ConversionError;
    fn try_from(value: Zoned) -> Result<Self, Self::Error> {
        let params = TimeParams {
            year: u16::try_from(value.year())
                .map_err(|_e| ConversionError(ConversionErrorInner::InvalidComponent))?,
            month: u8::try_from(value.month())
                .map_err(|_e| ConversionError(ConversionErrorInner::InvalidComponent))?,
            day: u8::try_from(value.day())
                .map_err(|_e| ConversionError(ConversionErrorInner::InvalidComponent))?,
            hour: u8::try_from(value.hour())
                .map_err(|_e| ConversionError(ConversionErrorInner::InvalidComponent))?,
            minute: u8::try_from(value.minute())
                .map_err(|_e| ConversionError(ConversionErrorInner::InvalidComponent))?,
            second: u8::try_from(value.second())
                .map_err(|_e| ConversionError(ConversionErrorInner::InvalidComponent))?,
            nanosecond: u32::try_from(value.subsec_nanosecond())
                .map_err(|_e| ConversionError(ConversionErrorInner::InvalidComponent))?,
            time_zone: Some(
                i16::try_from(value.offset().seconds() / 60 /* seconds per minute */)
                    .map_err(|_e| ConversionError(ConversionErrorInner::InvalidComponent))?,
            ),
            daylight: Default::default(),
        };
        Self::new(params).map_err(|e| ConversionError(ConversionErrorInner::InvalidUefiTime(e)))
    }
}
