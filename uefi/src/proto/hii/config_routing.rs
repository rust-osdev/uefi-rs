// SPDX-License-Identifier: MIT OR Apache-2.0

//! HII Configuration protocols.

use core::ptr;

use alloc::string::{String, ToString};
use uefi_macros::unsafe_protocol;
use uefi_raw::Char16;
use uefi_raw::protocol::hii::config::HiiConfigRoutingProtocol;

use crate::{CStr16, StatusExt};

/// The HII Configuration Routing Protocol.
///
/// # UEFI Spec Description
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
    pub fn export(&self) -> uefi::Result<String> {
        unsafe {
            let mut results: *const Char16 = ptr::null();
            (self.0.export_config)(&self.0, &mut results)
                .to_result_with_val(|| CStr16::from_ptr(results.cast()).to_string())
        }
    }
}
