///! Facilities for dealing with UEFI operation results.
///!
///! Almost all UEFI operations provide a status code as an output, which may
///! either indicate success, fatal failure, or non-fatal failure. In addition,
///! they may produce output, both in case of success and failure.
///!
///! We model this using an extended version of Rust's standard Result type,
///! whose successful path supports UEFI warnings and whose failing path can
///! report both an UEFI status code and extra data about the error.
///!
///! Convenience methods are also provided via extension traits to ease working
///! with this complex type in everyday usage.
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

/// Return type of most UEFI functions. Both success and error payloads are optional.
pub type Result<Output = (), ErrData = ()> =
    core::result::Result<Completion<Output>, Error<ErrData>>;

/// Extension trait for Result which helps dealing with UEFI's warnings
pub trait ResultExt<Output, ErrData: Debug> {
    /// Extract the UEFI status from this result
    fn status(&self) -> Status;

    /// Ignore warnings, keeping a trace of them in the logs
    fn log_warning(self) -> core::result::Result<Output, Error<ErrData>>;

    /// Expect success without warnings, panic otherwise
    fn unwrap_success(self) -> Output;

    /// Expect success without warnings, panic with provided message otherwise
    fn expect_success(self, msg: &str) -> Output;

    /// Expect error, panic with provided message otherwise, discarding output
    fn expect_error(self, msg: &str) -> Error<ErrData>;

    /// Transform the inner output, if any
    fn map_inner<Mapped>(self, f: impl FnOnce(Output) -> Mapped) -> Result<Mapped, ErrData>;

    /// Transform the ErrData value to ()
    fn discard_errdata(self) -> Result<Output>;

    /// Treat warnings as errors
    fn warning_as_error(self) -> core::result::Result<Output, Error<ErrData>>
    where
        ErrData: Default;
}

impl<Output, ErrData: Debug> ResultExt<Output, ErrData> for Result<Output, ErrData> {
    fn status(&self) -> Status {
        match self {
            Ok(c) => c.status(),
            Err(e) => e.status(),
        }
    }

    fn log_warning(self) -> core::result::Result<Output, Error<ErrData>> {
        self.map(Completion::log)
    }

    fn unwrap_success(self) -> Output {
        self.unwrap().unwrap()
    }

    fn expect_success(self, msg: &str) -> Output {
        self.expect(msg).expect(msg)
    }

    fn expect_error(self, msg: &str) -> Error<ErrData> {
        self.map(|completion| completion.status()).expect_err(msg)
    }

    fn map_inner<Mapped>(self, f: impl FnOnce(Output) -> Mapped) -> Result<Mapped, ErrData> {
        self.map(|completion| completion.map(f))
    }

    fn discard_errdata(self) -> Result<Output> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => Err(e.status().into()),
        }
    }

    fn warning_as_error(self) -> core::result::Result<Output, Error<ErrData>>
    where
        ErrData: Default,
    {
        match self.map(Completion::split) {
            Ok((Status::SUCCESS, res)) => Ok(res),
            Ok((s, _)) => Err(Error::new(s, Default::default())),
            Err(e) => Err(e),
        }
    }
}
