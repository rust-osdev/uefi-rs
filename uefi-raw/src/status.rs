use core::fmt::Debug;

newtype_enum! {
/// UEFI uses status codes in order to report successes, errors, and warnings.
///
/// The spec allows implementation-specific status codes, so the `Status`
/// constants are not a comprehensive list of all possible values.
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
    LOAD_ERROR              = Self::ERROR_BIT |  1,
    /// A parameter was incorrect.
    INVALID_PARAMETER       = Self::ERROR_BIT |  2,
    /// The operation is not supported.
    UNSUPPORTED             = Self::ERROR_BIT |  3,
    /// The buffer was not the proper size for the request.
    BAD_BUFFER_SIZE         = Self::ERROR_BIT |  4,
    /// The buffer is not large enough to hold the requested data.
    /// The required buffer size is returned in the appropriate parameter.
    BUFFER_TOO_SMALL        = Self::ERROR_BIT |  5,
    /// There is no data pending upon return.
    NOT_READY               = Self::ERROR_BIT |  6,
    /// The physical device reported an error while attempting the operation.
    DEVICE_ERROR            = Self::ERROR_BIT |  7,
    /// The device cannot be written to.
    WRITE_PROTECTED         = Self::ERROR_BIT |  8,
    /// A resource has run out.
    OUT_OF_RESOURCES        = Self::ERROR_BIT |  9,
    /// An inconstency was detected on the file system.
    VOLUME_CORRUPTED        = Self::ERROR_BIT | 10,
    /// There is no more space on the file system.
    VOLUME_FULL             = Self::ERROR_BIT | 11,
    /// The device does not contain any medium to perform the operation.
    NO_MEDIA                = Self::ERROR_BIT | 12,
    /// The medium in the device has changed since the last access.
    MEDIA_CHANGED           = Self::ERROR_BIT | 13,
    /// The item was not found.
    NOT_FOUND               = Self::ERROR_BIT | 14,
    /// Access was denied.
    ACCESS_DENIED           = Self::ERROR_BIT | 15,
    /// The server was not found or did not respond to the request.
    NO_RESPONSE             = Self::ERROR_BIT | 16,
    /// A mapping to a device does not exist.
    NO_MAPPING              = Self::ERROR_BIT | 17,
    /// The timeout time expired.
    TIMEOUT                 = Self::ERROR_BIT | 18,
    /// The protocol has not been started.
    NOT_STARTED             = Self::ERROR_BIT | 19,
    /// The protocol has already been started.
    ALREADY_STARTED         = Self::ERROR_BIT | 20,
    /// The operation was aborted.
    ABORTED                 = Self::ERROR_BIT | 21,
    /// An ICMP error occurred during the network operation.
    ICMP_ERROR              = Self::ERROR_BIT | 22,
    /// A TFTP error occurred during the network operation.
    TFTP_ERROR              = Self::ERROR_BIT | 23,
    /// A protocol error occurred during the network operation.
    PROTOCOL_ERROR          = Self::ERROR_BIT | 24,
    /// The function encountered an internal version that was
    /// incompatible with a version requested by the caller.
    INCOMPATIBLE_VERSION    = Self::ERROR_BIT | 25,
    /// The function was not performed due to a security violation.
    SECURITY_VIOLATION      = Self::ERROR_BIT | 26,
    /// A CRC error was detected.
    CRC_ERROR               = Self::ERROR_BIT | 27,
    /// Beginning or end of media was reached
    END_OF_MEDIA            = Self::ERROR_BIT | 28,
    /// The end of the file was reached.
    END_OF_FILE             = Self::ERROR_BIT | 31,
    /// The language specified was invalid.
    INVALID_LANGUAGE        = Self::ERROR_BIT | 32,
    /// The security status of the data is unknown or compromised and
    /// the data must be updated or replaced to restore a valid security status.
    COMPROMISED_DATA        = Self::ERROR_BIT | 33,
    /// There is an address conflict address allocation
    IP_ADDRESS_CONFLICT     = Self::ERROR_BIT | 34,
    /// A HTTP error occurred during the network operation.
    HTTP_ERROR              = Self::ERROR_BIT | 35,
}}

impl Status {
    /// Bit indicating that an UEFI status code is an error.
    pub const ERROR_BIT: usize = 1 << (core::mem::size_of::<usize>() * 8 - 1);

    /// Returns true if status code indicates success.
    #[inline]
    #[must_use]
    pub fn is_success(self) -> bool {
        self == Status::SUCCESS
    }

    /// Returns true if status code indicates a warning.
    #[inline]
    #[must_use]
    pub fn is_warning(self) -> bool {
        (self != Status::SUCCESS) && (self.0 & Self::ERROR_BIT == 0)
    }

    /// Returns true if the status code indicates an error.
    #[inline]
    #[must_use]
    pub const fn is_error(self) -> bool {
        self.0 & Self::ERROR_BIT != 0
    }
}

impl core::fmt::Display for Status {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self, f)
    }
}
