use super::{Error, Result};
use core::fmt::Debug;

/// Bit indicating that an UEFI status code is an error
const ERROR_BIT: usize = 1 << (core::mem::size_of::<usize>() * 8 - 1);

newtype_enum! {
/// UEFI uses status codes in order to report successes, errors, and warnings.
///
/// Unfortunately, the spec allows and encourages implementation-specific
/// non-portable status codes. Therefore, these cannot be modeled as a Rust
/// enum, as injecting an unknown value in a Rust enum is undefined behaviour.
///
/// For lack of a better option, we therefore model them as a newtype of usize.
#[must_use]
pub enum Status: usize => {
    /// The operation completed successfully.
    SUCCESS                 =  0,

    /// The string contained characters that could not be rendered and were skipped.
    WARN_UNKNOWN_GLYPH      =  1,
    /// The handle was closed, but the file was not deleted.
    WARN_DELETE_FAILURE     =  2,
    /// The handle was closed, but the data to the file was not flushed properly.
    WARN_WRITE_FAILURE      =  3,
    /// The resulting buffer was too small, and the data was truncated.
    WARN_BUFFER_TOO_SMALL   =  4,
    /// The data has not been updated within the timeframe set by local policy.
    WARN_STALE_DATA         =  5,
    /// The resulting buffer contains UEFI-compliant file system.
    WARN_FILE_SYSTEM        =  6,
    /// The operation will be processed across a system reset.
    WARN_RESET_REQUIRED     =  7,

    /// The image failed to load.
    LOAD_ERROR              = ERROR_BIT |  1,
    /// A parameter was incorrect.
    INVALID_PARAMETER       = ERROR_BIT |  2,
    /// The operation is not supported.
    UNSUPPORTED             = ERROR_BIT |  3,
    /// The buffer was not the proper size for the request.
    BAD_BUFFER_SIZE         = ERROR_BIT |  4,
    /// The buffer is not large enough to hold the requested data.
    /// The required buffer size is returned in the appropriate parameter.
    BUFFER_TOO_SMALL        = ERROR_BIT |  5,
    /// There is no data pending upon return.
    NOT_READY               = ERROR_BIT |  6,
    /// The physical device reported an error while attempting the operation.
    DEVICE_ERROR            = ERROR_BIT |  7,
    /// The device cannot be written to.
    WRITE_PROTECTED         = ERROR_BIT |  8,
    /// A resource has run out.
    OUT_OF_RESOURCES        = ERROR_BIT |  9,
    /// An inconstency was detected on the file system.
    VOLUME_CORRUPTED        = ERROR_BIT | 10,
    /// There is no more space on the file system.
    VOLUME_FULL             = ERROR_BIT | 11,
    /// The device does not contain any medium to perform the operation.
    NO_MEDIA                = ERROR_BIT | 12,
    /// The medium in the device has changed since the last access.
    MEDIA_CHANGED           = ERROR_BIT | 13,
    /// The item was not found.
    NOT_FOUND               = ERROR_BIT | 14,
    /// Access was denied.
    ACCESS_DENIED           = ERROR_BIT | 15,
    /// The server was not found or did not respond to the request.
    NO_RESPONSE             = ERROR_BIT | 16,
    /// A mapping to a device does not exist.
    NO_MAPPING              = ERROR_BIT | 17,
    /// The timeout time expired.
    TIMEOUT                 = ERROR_BIT | 18,
    /// The protocol has not been started.
    NOT_STARTED             = ERROR_BIT | 19,
    /// The protocol has already been started.
    ALREADY_STARTED         = ERROR_BIT | 20,
    /// The operation was aborted.
    ABORTED                 = ERROR_BIT | 21,
    /// An ICMP error occurred during the network operation.
    ICMP_ERROR              = ERROR_BIT | 22,
    /// A TFTP error occurred during the network operation.
    TFTP_ERROR              = ERROR_BIT | 23,
    /// A protocol error occurred during the network operation.
    PROTOCOL_ERROR          = ERROR_BIT | 24,
    /// The function encountered an internal version that was
    /// incompatible with a version requested by the caller.
    INCOMPATIBLE_VERSION    = ERROR_BIT | 25,
    /// The function was not performed due to a security violation.
    SECURITY_VIOLATION      = ERROR_BIT | 26,
    /// A CRC error was detected.
    CRC_ERROR               = ERROR_BIT | 27,
    /// Beginning or end of media was reached
    END_OF_MEDIA            = ERROR_BIT | 28,
    /// The end of the file was reached.
    END_OF_FILE             = ERROR_BIT | 31,
    /// The language specified was invalid.
    INVALID_LANGUAGE        = ERROR_BIT | 32,
    /// The security status of the data is unknown or compromised and
    /// the data must be updated or replaced to restore a valid security status.
    COMPROMISED_DATA        = ERROR_BIT | 33,
    /// There is an address conflict address allocation
    IP_ADDRESS_CONFLICT     = ERROR_BIT | 34,
    /// A HTTP error occurred during the network operation.
    HTTP_ERROR              = ERROR_BIT | 35,
}}

impl Status {
    /// Returns true if status code indicates success.
    #[inline]
    pub fn is_success(self) -> bool {
        self == Status::SUCCESS
    }

    /// Returns true if status code indicates a warning.
    #[inline]
    pub fn is_warning(self) -> bool {
        (self != Status::SUCCESS) && (self.0 & ERROR_BIT == 0)
    }

    /// Returns true if the status code indicates an error.
    #[inline]
    pub const fn is_error(self) -> bool {
        self.0 & ERROR_BIT != 0
    }

    /// Converts this status code into a result with a given value.
    #[inline]
    pub fn into_with_val<T>(self, val: impl FnOnce() -> T) -> Result<T, ()> {
        if self.is_success() {
            Ok(val())
        } else {
            Err(self.into())
        }
    }

    /// Converts this status code into a result with a given error payload
    #[inline]
    pub fn into_with_err<ErrData: Debug>(
        self,
        err: impl FnOnce(Status) -> ErrData,
    ) -> Result<(), ErrData> {
        if self.is_success() {
            Ok(())
        } else {
            Err(Error::new(self, err(self)))
        }
    }

    /// Convert this status code into a result with a given value and error payload
    #[inline]
    pub fn into_with<T, ErrData: Debug>(
        self,
        val: impl FnOnce() -> T,
        err: impl FnOnce(Status) -> ErrData,
    ) -> Result<T, ErrData> {
        if self.is_success() {
            Ok(val())
        } else {
            Err(Error::new(self, err(self)))
        }
    }
}

// An UEFI status is equivalent to a Result with no data or error payload
impl From<Status> for Result<(), ()> {
    #[inline]
    fn from(status: Status) -> Result<(), ()> {
        status.into_with(|| (), |_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_to_result() {
        assert!(Result::from(Status::SUCCESS).is_ok());
        assert!(Result::from(Status::WARN_DELETE_FAILURE).is_err());
        assert!(Result::from(Status::BUFFER_TOO_SMALL).is_err());

        assert_eq!(Status::SUCCESS.into_with_val(|| 123).unwrap(), 123);
        assert!(Status::WARN_DELETE_FAILURE.into_with_val(|| 123).is_err());
        assert!(Status::BUFFER_TOO_SMALL.into_with_val(|| 123).is_err());

        assert!(Status::SUCCESS.into_with_err(|_| 123).is_ok());
        assert_eq!(
            *Status::WARN_DELETE_FAILURE
                .into_with_err(|_| 123)
                .unwrap_err()
                .data(),
            123
        );
        assert_eq!(
            *Status::BUFFER_TOO_SMALL
                .into_with_err(|_| 123)
                .unwrap_err()
                .data(),
            123
        );

        assert_eq!(Status::SUCCESS.into_with(|| 123, |_| 456).unwrap(), 123);
        assert_eq!(
            *Status::WARN_DELETE_FAILURE
                .into_with(|| 123, |_| 456)
                .unwrap_err()
                .data(),
            456
        );
        assert_eq!(
            *Status::BUFFER_TOO_SMALL
                .into_with(|| 123, |_| 456)
                .unwrap_err()
                .data(),
            456
        );
    }
}
