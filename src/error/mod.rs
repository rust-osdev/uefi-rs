use core::result;
use log::warn;

/// Definition of UEFI's standard status codes
mod status;
pub use self::status::Status;

/// This type is used when an UEFI operation has completed, but some non-fatal
/// problems may have been encountered along the way
#[must_use]
pub enum Completion<T> {
    /// The operation completed without problems
    Success(T),

    /// The operation completed, but some non-fatal issues were encountered
    Warning(T, Status),
}

impl<T> Completion<T> {
    /// Split the complation into a (status, value) pair
    pub fn split(self) -> (T, Status) {
        match self {
            Completion::Success(res) => (res, Status::SUCCESS),
            Completion::Warning(res, stat) => (res, stat),
        }
    }

    /// Access the inner value, logging the warning if there is any
    pub fn unwrap(self) -> T {
        match self {
            Completion::Success(res) => res,
            Completion::Warning(res, stat) => {
                warn!("Encountered UEFI warning {:?}", stat);
                res
            },
        }
    }

    /// Transform the inner value without unwrapping the Completion
    pub fn map<U>(self, f: impl Fn(T) -> U) -> Completion<U> {
        match self {
            Completion::Success(res) => Completion::Success(f(res)),
            Completion::Warning(res, stat) => Completion::Warning(f(res), stat),
        }
    }

    /// Merge this completion with a success or warning status
    ///
    /// Since this type only has storage for one warning, if two warnings must
    /// be stored, one of them will be spilled into the logs.
    ///
    pub fn with_warning(self, extra_stat: Status) -> Self {
        assert!(!extra_stat.is_error(), "Completions do not handle error status");
        match self {
            Completion::Success(res) => {
                if extra_stat.is_success() {
                    Completion::Success(res)
                } else {
                    Completion::Warning(res, extra_stat)
                }
            }
            Completion::Warning(res, stat) => {
                if extra_stat.is_success() {
                    Completion::Warning(res, stat)
                } else {
                    warn!("Encountered UEFI warning {:?}", stat);
                    Completion::Warning(res, extra_stat)
                }
            }
        }
    }

}

impl<T> From<T> for Completion<T> {
    fn from(res: T) -> Self {
        Completion::Success(res)
    }
}

/// Return type of many UEFI functions.
pub type Result<T> = result::Result<Completion<T>, Status>;
