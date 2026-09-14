// SPDX-License-Identifier: MIT OR Apache-2.0

//! HII Configuration protocols.

use core::ptr;

use alloc::string::{String, ToString};
use uefi_macros::unsafe_protocol;
use uefi_raw::Char16;
use uefi_raw::protocol::hii::config::HiiConfigRoutingProtocol;

use crate::data_types::PoolString;
use crate::{Status, StatusExt};

/// The HII Configuration Routing Protocol.
///
/// # UEFI Specification
///
/// The EFI HII Configuration Routing Protocol manages the movement of configuration
/// data from drivers to configuration applications. It then serves as the single point
/// to receive configuration information from configuration applications, routing the results
/// to the appropriate drivers.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(HiiConfigRoutingProtocol::GUID)]
pub struct HiiConfigRouting(HiiConfigRoutingProtocol);
impl HiiConfigRouting {
    /// Request the current configuration for the entirety of the current HII database and
    /// return the data as string in multi configuration string format.
    ///
    /// Use `super::config_str::MultiConfigurationStringIter` to parse the returned `String`.
    ///
    /// # Errors
    ///
    /// * [`Status::NOT_FOUND`] - The firmware reported success but did not
    ///   provide a result string.
    pub fn export(&self) -> uefi::Result<String> {
        let mut results: *mut Char16 = ptr::null_mut();
        // SAFETY: The memory is valid.
        unsafe { (self.0.export_config)(&self.0, &mut results) }.to_result()?;
        if results.is_null() {
            return Err(Status::NOT_FOUND.into());
        }
        // SAFETY: The firmware allocated the NUL-terminated string from the
        // pool; `PoolString` frees it.
        let results = unsafe { PoolString::new(results.cast()) }?;
        Ok(results.to_string())
    }
}
