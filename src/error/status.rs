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

/// Bit indicating that a status code is an error
const ERROR_BIT: usize = 1 << (core::mem::size_of::<usize>()*8 - 1);

/// The operation completed successfully.
pub const SUCCESS: Status = Status(0);

/// Macro to make implementation of error status codes easier
macro_rules! error_codes {
    ( $( $status:ident => $value:expr, $docstring:expr ),* ) => {
        $(
            #[doc = $docstring]
            #[allow(unused)]
            pub const $status: Status = Status($value | ERROR_BIT);
        )*
    }
}
//
error_codes! {
    LOAD_ERROR              =>  1, "The image failed to load.",

    INVALID_PARAMETER       =>  2, "A parameter was incorrect.",

    UNSUPPORTED             =>  3, "The operation is not supported.",

    BAD_BUFFER_SIZE         =>  4, "The buffer was not the proper size for the \
                                    request.",

    BUFFER_TOO_SMALL        =>  5, "The buffer is not large enough to hold the \
                                    requested data. The required buffer size \
                                    is returned in the appropriate parameter.",

    NOT_READY               =>  6, "There is no data pending upon return.",

    DEVICE_ERROR            =>  7, "The physical device reported an error \
                                    while attempting the operation.",

    WRITE_PROTECTED         =>  8, "The device cannot be written to.",

    OUT_OF_RESOURCES        =>  9, "A resource has run out.",

    VOLUME_CORRUPTED        => 10, "An inconstancy was detected on the file \
                                    system causing the operating to fail.",

    VOLUME_FULL             => 11, "There is no more space on the file system.",

    NO_MEDIA                => 12, "The device does not contain any medium to \
                                    perform the operation.",

    MEDIA_CHANGED           => 13, "The medium in the device has changed since \
                                    the last access.",

    NOT_FOUND               => 14, "The item was not found.",

    ACCESS_DENIED           => 15, "Access was denied.",

    NO_RESPONSE             => 16, "The server was not found or did not \
                                    respond to the request.",

    NO_MAPPING              => 17, "A mapping to a device does not exist.",

    TIMEOUT                 => 18, "The timeout time expired.",

    NOT_STARTED             => 19, "The protocol has not been started.",

    ALREADY_STARTED         => 20, "The protocol has already been started.",

    ABORTED                 => 21, "The operation was aborted.",

    ICMP_ERROR              => 22, "An ICMP error occurred during the network \
                                    operation.",

    TFTP_ERROR              => 23, "A TFTP error occurred during the network \
                                    operation.",

    PROTOCOL_ERROR          => 24, "A protocol error occurred during the \
                                    network operation.",

    INCOMPATIBLE_VERSION    => 25, "The function encountered an internal \
                                    version that was incompatible with a \
                                    version requested by the caller.",

    SECURITY_VIOLATION      => 26, "The function was not performed due to a \
                                    security violation.",

    CRC_ERROR               => 27, "A CRC error was detected.",

    END_OF_MEDIA            => 28, "Beginning or end of media was reached",

    END_OF_FILE             => 31, "The end of the file was reached.",

    INVALID_LANGUAGE        => 32, "The language specified was invalid.",

    COMPROMISED_DATA        => 33, "The security status of the data is unknown \
                                    or compromised and the data must be \
                                    updated or replaced to restore a valid \
                                    security status.",

    IP_ADDRESS_CONFLICT     => 34, "There is an address conflict address \
                                    allocation",

    HTTP_ERROR              => 35, "A HTTP error occurred during the network \
                                    operation."
}

/// Macro to make implementation of warning status codes easier
macro_rules! warning_codes {
    ( $( $status:ident => $value:expr, $docstring:expr ),* ) => {
        $(
            #[doc = $docstring]
            #[allow(unused)]
            pub const $status: Status = Status($value);
        )*
    }
}
//
warning_codes! {
    WARN_UNKNOWN_GLYPH      =>  1, "The string contained characters that the \
                                    device could not render and were skipped.",

    WARN_DELETE_FAILURE     =>  2, "The handle was closed, but the file was \
                                    not deleted.",

    WARN_WRITE_FAILURE      =>  3, "The handle was closed, but the data to the \
                                    file was not flushed properly.",

    WARN_BUFFER_TOO_SMALL   =>  4, "The resulting buffer was too small, and \
                                    the data was truncated to the buffer size.",

    WARN_STALE_DATA         =>  5, "The data has not been updated within the \
                                    timeframe set by local policy for this \
                                    type of data.",

    WARN_FILE_SYSTEM        =>  6, "The resulting buffer contains \
                                    UEFI-compliant file system.",

    WARN_RESET_REQUIRED     =>  7, "The operation will be processed across a \
                                    system reset."
}

impl Status {
    /// Returns true if status code indicates success.
    #[inline]
    pub fn is_success(self) -> bool {
        self == SUCCESS
    }

    /// Returns true if status code indicates a warning.
    #[inline]
    pub fn is_warning(self) -> bool {
        (self != SUCCESS) && (self.0 & ERROR_BIT == 0)
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
        SUCCESS
    }
}

impl From<ucs2::Error> for Status {
    fn from(other: ucs2::Error) -> Self {
        use ucs2::Error;
        match other {
            Error::InvalidData => INVALID_PARAMETER,
            Error::BufferUnderflow => BAD_BUFFER_SIZE,
            Error::BufferOverflow => BUFFER_TOO_SMALL,
            Error::MultiByte => UNSUPPORTED,
        }
    }
}
