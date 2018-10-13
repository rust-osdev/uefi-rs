/// Definition of UEFI's standard status codes
mod status;
pub use self::status::Status;

/// Completions are used to model operations which have completed, but may have
/// encountered non-fatal errors ("warnings") along the way
mod completion;
pub use self::completion::Completion;

/// Return type of many UEFI functions.
pub type Result<T> = core::result::Result<Completion<T>, Status>;

/// Extension trait for Result which helps dealing with UEFI's warnings
pub trait ResultExt<T> {
    /// Treat warnings as errors
    fn warn_error(self) -> core::result::Result<T, Status>;

    /// Ignore warnings, keepint a trace of them in the logs
    fn warn_log(self) -> core::result::Result<T, Status>;

    /// Expect success without warnings, panic otherwise
    fn warn_unwrap(self) -> T;

    /// Expect success without warnings, panic with provided message otherwise
    fn warn_expect(self, msg: &str) -> T;
}

impl<T> ResultExt<T> for Result<T> {
    fn warn_error(self) -> core::result::Result<T, Status> {
        match self {
            Ok(Completion::Success(v)) => Ok(v),
            Ok(Completion::Warning(_, s)) => Err(s),
            Err(s) => Err(s),
        }
    }

    fn warn_log(self) -> core::result::Result<T, Status> {
        self.map(|completion| completion.value())
    }

    fn warn_unwrap(self) -> T {
        self.unwrap().unwrap()
    }

    fn warn_expect(self, msg: &str) -> T {
        self.expect(msg).expect(msg)
    }
}
