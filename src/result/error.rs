use super::Status;
use core::fmt::Debug;

/// Errors emitted from UEFI entry point must propagate erronerous UEFI statuses,
/// and may optionally propagate additional entry point-specific data.
#[derive(Debug)]
pub struct Error<Data: Debug = ()> {
    status: Status,
    data: Data,
}

impl<Data: Debug> Error<Data> {
    
    /// Construct an error from an inner status and payload data
    pub fn new(status: Status, data: Data) -> Self {
        Self { status, data }
    }

    /// Return the inner status for this error
    pub fn status(&self) -> Status {
        self.status
    }

    /// Return the data for this error
    pub fn data(&self) -> &Data {
        &self.data
    }

    /// Split this error into its inner status and error data
    pub fn split(self) -> (Status, Data) {
        (self.status, self.data)
    }
}

// Errors without payloads can be autogenerated from statuses

impl From<Status> for Error<()> {
    fn from(status: Status) -> Self {
        Self { status, data: () }
    }
}

// FIXME: This conversion will go away along with usage of the ucs2 crate

impl From<ucs2::Error> for Error<()> {
    fn from(other: ucs2::Error) -> Self {
        Status::from(other).into()
    }
}
