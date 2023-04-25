use super::{Error, Result};
use core::fmt::Debug;

pub use uefi_raw::Status;

/// Extension trait which provides some convenience methods for [`Status`].
pub trait StatusExt {
    /// Converts this status code into a [`uefi::Result`].
    ///
    /// If the status does not indicate success, the status representing the specific error
    /// code is embedded into the `Err` variant of type [`uefi::Error`].
    fn to_result(self) -> Result;

    /// Converts this status code into a [`uefi::Result`] with a given `Ok` value.
    ///
    /// If the status does not indicate success, the status representing the specific error
    /// code is embedded into the `Err` variant of type [`uefi::Error`].
    fn to_result_with_val<T>(self, val: impl FnOnce() -> T) -> Result<T, ()>;

    /// Converts this status code into a [`uefi::Result`] with a given `Err` payload.
    ///
    /// If the status does not indicate success, the status representing the specific error
    /// code is embedded into the `Err` variant of type [`uefi::Error`].
    fn to_result_with_err<ErrData: Debug>(
        self,
        err: impl FnOnce(Status) -> ErrData,
    ) -> Result<(), ErrData>;

    /// Convert this status code into a result with a given `Ok` value and `Err` payload.
    ///
    /// If the status does not indicate success, the status representing the specific error
    /// code is embedded into the `Err` variant of type [`uefi::Error`].
    fn to_result_with<T, ErrData: Debug>(
        self,
        val: impl FnOnce() -> T,
        err: impl FnOnce(Status) -> ErrData,
    ) -> Result<T, ErrData>;
}

impl StatusExt for Status {
    #[inline]
    fn to_result(self) -> Result {
        if self.is_success() {
            Ok(())
        } else {
            Err(self.into())
        }
    }

    #[inline]
    fn to_result_with_val<T>(self, val: impl FnOnce() -> T) -> Result<T, ()> {
        if self.is_success() {
            Ok(val())
        } else {
            Err(self.into())
        }
    }

    #[inline]
    fn to_result_with_err<ErrData: Debug>(
        self,
        err: impl FnOnce(Status) -> ErrData,
    ) -> Result<(), ErrData> {
        if self.is_success() {
            Ok(())
        } else {
            Err(Error::new(self, err(self)))
        }
    }

    #[inline]
    fn to_result_with<T, ErrData: Debug>(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_to_result() {
        assert!(Status::SUCCESS.to_result().is_ok());
        assert!(Status::WARN_DELETE_FAILURE.to_result().is_err());
        assert!(Status::BUFFER_TOO_SMALL.to_result().is_err());

        assert_eq!(Status::SUCCESS.to_result_with_val(|| 123).unwrap(), 123);
        assert!(Status::WARN_DELETE_FAILURE
            .to_result_with_val(|| 123)
            .is_err());
        assert!(Status::BUFFER_TOO_SMALL.to_result_with_val(|| 123).is_err());

        assert!(Status::SUCCESS.to_result_with_err(|_| 123).is_ok());
        assert_eq!(
            *Status::WARN_DELETE_FAILURE
                .to_result_with_err(|_| 123)
                .unwrap_err()
                .data(),
            123
        );
        assert_eq!(
            *Status::BUFFER_TOO_SMALL
                .to_result_with_err(|_| 123)
                .unwrap_err()
                .data(),
            123
        );

        assert_eq!(
            Status::SUCCESS.to_result_with(|| 123, |_| 456).unwrap(),
            123
        );
        assert_eq!(
            *Status::WARN_DELETE_FAILURE
                .to_result_with(|| 123, |_| 456)
                .unwrap_err()
                .data(),
            456
        );
        assert_eq!(
            *Status::BUFFER_TOO_SMALL
                .to_result_with(|| 123, |_| 456)
                .unwrap_err()
                .data(),
            456
        );
    }
}
