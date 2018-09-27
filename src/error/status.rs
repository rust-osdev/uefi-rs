// The error codes are unportable, but that's how the spec defines them.
#![allow(clippy::enum_clike_unportable_variant)]

use super::Result;
use core::ops;
use ucs2;

/// UEFI uses status codes in order to report successes, errors, and warnings.
///
/// Unfortunately, the spec allows and encourages implementation-specific
/// non-portable status codes. Therefore, these cannot be modeled as a Rust
/// enum, as injecting an unknown value in a Rust enum is undefined behaviour.
///
/// For lack of a better option, we therefore model them as a newtype of usize.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
#[must_use]
pub struct Status(usize);

/// Macro to make implementation of status codes easier
macro_rules! status_codes {
    (   $(  $(#[$attr:meta])*
            $status:ident = $code:expr, )*
    ) => {
        #[allow(unused)]
        impl Status {
            $(  $(#[$attr])*
                pub const $status: Status = Status($code); )*
        }
    }
}
//
status_codes! {
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
}

/// Bit indicating that a status code is an error
const ERROR_BIT: usize = 1 << (core::mem::size_of::<usize>() * 8 - 1);

/// Macro to make implementation of error codes easier
macro_rules! error_codes {
    (   $(  $(#[$attr:meta])*
            $status:ident = $error_code:expr, )*
    ) => {
        status_codes! { $(
            $(#[$attr])*
            $status = $error_code | ERROR_BIT,
        )* }
    }
}
//
error_codes! {
    /// The image failed to load.
    LOAD_ERROR              =  1,
    /// A parameter was incorrect.
    INVALID_PARAMETER       =  2,
    /// The operation is not supported.
    UNSUPPORTED             =  3,
    /// The buffer was not the proper size for the request.
    BAD_BUFFER_SIZE         =  4,
    /// The buffer is not large enough to hold the requested data.
    /// The required buffer size is returned in the appropriate parameter.
    BUFFER_TOO_SMALL        =  5,
    /// There is no data pending upon return.
    NOT_READY               =  6,
    /// The physical device reported an error while attempting the operation.
    DEVICE_ERROR            =  7,
    /// The device cannot be written to.
    WRITE_PROTECTED         =  8,
    /// A resource has run out.
    OUT_OF_RESOURCES        =  9,
    /// An inconstency was detected on the file system.
    VOLUME_CORRUPTED        = 10,
    /// There is no more space on the file system.
    VOLUME_FULL             = 11,
    /// The device does not contain any medium to perform the operation.
    NO_MEDIA                = 12,
    /// The medium in the device has changed since the last access.
    MEDIA_CHANGED           = 13,
    /// The item was not found.
    NOT_FOUND               = 14,
    /// Access was denied.
    ACCESS_DENIED           = 15,
    /// The server was not found or did not respond to the request.
    NO_RESPONSE             = 16,
    /// A mapping to a device does not exist.
    NO_MAPPING              = 17,
    /// The timeout time expired.
    TIMEOUT                 = 18,
    /// The protocol has not been started.
    NOT_STARTED             = 19,
    /// The protocol has already been started.
    ALREADY_STARTED         = 20,
    /// The operation was aborted.
    ABORTED                 = 21,
    /// An ICMP error occurred during the network operation.
    ICMP_ERROR              = 22,
    /// A TFTP error occurred during the network operation.
    TFTP_ERROR              = 23,
    /// A protocol error occurred during the network operation.
    PROTOCOL_ERROR          = 24,
    /// The function encountered an internal version that was
    /// incompatible with a version requested by the caller.
    INCOMPATIBLE_VERSION    = 25,
    /// The function was not performed due to a security violation.
    SECURITY_VIOLATION      = 26,
    /// A CRC error was detected.
    CRC_ERROR               = 27,
    /// Beginning or end of media was reached
    END_OF_MEDIA            = 28,
    /// The end of the file was reached.
    END_OF_FILE             = 31,
    /// The language specified was invalid.
    INVALID_LANGUAGE        = 32,
    /// The security status of the data is unknown or compromised and
    /// the data must be updated or replaced to restore a valid security status.
    COMPROMISED_DATA        = 33,
    /// There is an address conflict address allocation
    IP_ADDRESS_CONFLICT     = 34,
    /// A HTTP error occurred during the network operation.
    HTTP_ERROR              = 35,
}

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
    pub fn is_error(self) -> bool {
        self.0 & ERROR_BIT != 0
    }

    /// Converts this status code into a result with a given value.
    #[inline]
    pub fn into_with<T, F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> T,
    {
        // FIXME: Is that the best way to handle warnings?
        if self.is_success() {
            Ok(f())
        } else {
            Err(self)
        }
    }
}

impl Into<Result<()>> for Status {
    #[inline]
    fn into(self) -> Result<()> {
        self.into_with(|| ())
    }
}

impl ops::Try for Status {
    type Ok = ();
    type Error = Status;

    fn into_result(self) -> Result<()> {
        self.into()
    }

    fn from_error(error: Self::Error) -> Self {
        error
    }

    fn from_ok(_: Self::Ok) -> Self {
        Status::SUCCESS
    }
}

impl From<ucs2::Error> for Status {
    fn from(other: ucs2::Error) -> Self {
        use ucs2::Error;
        match other {
            Error::InvalidData => Status::INVALID_PARAMETER,
            Error::BufferUnderflow => Status::BAD_BUFFER_SIZE,
            Error::BufferOverflow => Status::BUFFER_TOO_SMALL,
            Error::MultiByte => Status::UNSUPPORTED,
        }
    }
}
