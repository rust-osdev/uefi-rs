use core::result;

/// Definition of UEFI's standard status codes
pub mod status;
pub use self::status::Status;

/// Return type of many UEFI functions.
pub type Result<T> = result::Result<T, Status>;
