//! Shim lock protocol.

use crate::proto::Protocol;
use crate::result::Error;
use crate::{unsafe_guid, Result, Status};
use core::convert::TryInto;

/// The Shim lock protocol.
///
/// This protocol is not part of the UEFI specification, but is
/// installed by the [Shim bootloader](https://github.com/rhboot/shim)
/// which is commonly used by Linux distributions to support UEFI
/// Secure Boot. Shim is built with an embedded certificate that is
/// used to validate another EFI application before running it. That
/// application may itself be a bootloader that needs to validate
/// another EFI application before running it, and the shim lock
/// protocol exists to support that.
#[repr(C)]
#[unsafe_guid("605dab50-e046-4300-abb6-3dd810dd8b23")]
#[derive(Protocol)]
pub struct ShimLock {
    verify: extern "sysv64" fn(buffer: *const u8, size: u32) -> Status,
}

impl ShimLock {
    /// Verify that an EFI application is signed by the certificate
    /// embedded in shim.
    ///
    /// The buffer's size must fit in a `u32`; if that condition is not
    /// met then a `BAD_BUFFER_SIZE` error will be returned and the shim
    /// lock protocol will not be called.
    pub fn verify(&self, buffer: &[u8]) -> Result {
        let size: u32 = buffer
            .len()
            .try_into()
            .map_err(|_| Error::from(Status::BAD_BUFFER_SIZE))?;
        (self.verify)(buffer.as_ptr(), size).into()
    }
}
