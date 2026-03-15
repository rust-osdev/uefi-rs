// SPDX-License-Identifier: MIT OR Apache-2.0

//! Provides common helpers for the integration and conversion with
//! different time crates from the ecosystem.

use crate::runtime::TimeError;
use core::error::Error;
use core::fmt;
use core::fmt::{Display, Formatter};

/// Opaque error type indicating a UEFI [`Time`] could not be converted.
///
/// [`Time`]: super::Time
#[derive(Debug)]
pub struct TimeConversionError(pub(super) ConversionErrorInner);

impl Display for TimeConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Time conversion error: {}", self.0)
    }
}

impl Error for TimeConversionError {}

#[derive(Debug)]
pub(super) enum ConversionErrorInner {
    /// Invalid component.
    InvalidComponent,
    /// Invalid UEFI time: [`Time::is_valid`] reported an error.
    ///
    /// [`Time::is_valid`]: super::Time::is_valid
    InvalidUefiTime(TimeError),
    /// A timezone was required for the conversion, but the UEFI time indicates
    /// [`Time::UNSPECIFIED_TIMEZONE`].
    ///
    /// [`Time::UNSPECIFIED_TIMEZONE`]: super::Time::UNSPECIFIED_TIMEZONE
    UnspecifiedTimezone,
    /// Errors raised in the [`time`] crate.
    #[cfg(feature = "time03")]
    TimeCrateError(time::Error),
}

impl Display for ConversionErrorInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidComponent => write!(f, "Invalid component"),
            Self::InvalidUefiTime(e) => write!(f, "Invalid UEFI time: {e}"),
            Self::UnspecifiedTimezone => write!(f, "Unspecified timezone"),
            #[cfg(feature = "time03")]
            Self::TimeCrateError(e) => write!(f, "Time crate error: {}", e),
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
        }
    }
}
