use core::fmt::Debug;

/// `Completion`s are used to model operations which have completed, but may
/// have encountered non-fatal errors ("warnings") along the way
mod completion;
pub use self::completion::Completion;

/// The error type that we use, essentially a status code + optional additional data
mod error;
pub use self::error::Error;

/// Definition of UEFI's standard status codes
mod status;
pub use self::status::Status;

/// Return type of many UEFI functions.
pub type Result<T, ErrData = ()> = core::result::Result<Completion<T>, Error<ErrData>>;

/// Extension trait for Result which helps dealing with UEFI's warnings
pub trait ResultExt<T, ErrData: Debug> {
    /// Extract the UEFI status from this result
    fn status(&self) -> Status;

    /// Ignore warnings, keeping a trace of them in the logs
    fn log_warning(self) -> core::result::Result<T, Error<ErrData>>;

    /// Expect success without warnings, panic otherwise
    fn unwrap_success(self) -> T;

    /// Expect success without warnings, panic with provided message otherwise
    fn expect_success(self, msg: &str) -> T;

    /// Transform the inner output, if any
    fn map_inner<U>(self, f: impl FnOnce(T) -> U) -> Result<U, ErrData>;
}

/// Extension trait for results with no error payload
pub trait ResultExt2<T> {
    /// Treat warnings as errors
    fn warning_as_error(self) -> core::result::Result<T, Error<()>>;
}

impl<T, ErrData: Debug> ResultExt<T, ErrData> for Result<T, ErrData> {
    fn status(&self) -> Status {
        match self {
            Ok(c) => c.status(),
            Err(e) => e.status(),
        }
    }

    fn log_warning(self) -> core::result::Result<T, Error<ErrData>> {
        self.map(|completion| completion.log())
    }

    fn unwrap_success(self) -> T {
        self.unwrap().unwrap()
    }

    fn expect_success(self, msg: &str) -> T {
        self.expect(msg).expect(msg)
    }

    fn map_inner<U>(self, f: impl FnOnce(T) -> U) -> Result<U, ErrData> {
        self.map(|completion| completion.map(f))
    }
}

impl<T> ResultExt2<T> for Result<T, ()> {
    fn warning_as_error(self) -> core::result::Result<T, Error<()>> {
        match self.map(|comp| comp.split()) {
            Ok((Status::SUCCESS, res)) => Ok(res),
            Ok((s, _)) => Err(s.into()),
            Err(e) => Err(e),
        }
    }
}
