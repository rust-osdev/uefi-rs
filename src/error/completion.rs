use super::Status;
use log::warn;

/// This type is used when an UEFI operation has completed, but some non-fatal
/// problems may have been encountered along the way
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Completion<T> {
    /// The operation completed without problems
    Success(T),

    /// The operation completed, but some non-fatal issues were encountered
    Warning(T, Status),
}

impl<T> Completion<T> {
    /// Split the completion into a (status, value) pair
    pub fn split(self) -> (T, Status) {
        match self {
            Completion::Success(res) => (res, Status::SUCCESS),
            Completion::Warning(res, stat) => (res, stat),
        }
    }

    /// Access the inner value, logging the warning if there is any
    pub fn log(self) -> T {
        match self {
            Completion::Success(res) => res,
            Completion::Warning(res, stat) => {
                log_warning(stat);
                res
            }
        }
    }

    /// Assume that no warning occured, panic if not
    pub fn unwrap(self) -> T {
        match self {
            Completion::Success(res) => res,
            Completion::Warning(_, w) => {
                unwrap_failed("Called `Completion::unwrap()` on a `Warning` value", w)
            }
        }
    }

    /// Assume that no warning occured, panic with provided message if not
    pub fn expect(self, msg: &str) -> T {
        match self {
            Completion::Success(res) => res,
            Completion::Warning(_, w) => unwrap_failed(msg, w),
        }
    }

    /// Transform the inner value without unwrapping the Completion
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Completion<U> {
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
    pub fn with_status(self, extra_status: Status) -> Self {
        if extra_status.is_success() {
            self
        } else {
            Completion::Warning(self.log(), extra_status)
        }
    }
}

impl<T> From<T> for Completion<T> {
    fn from(res: T) -> Self {
        Completion::Success(res)
    }
}

// These are separate functions to reduce the code size of the methods
#[inline(never)]
#[cold]
fn unwrap_failed(msg: &str, warning: Status) -> ! {
    panic!("{}: {:?}", msg, warning)
}

#[inline(never)]
#[cold]
fn log_warning(warning: Status) {
    warn!("Encountered UEFI warning: {:?}", warning)
}
