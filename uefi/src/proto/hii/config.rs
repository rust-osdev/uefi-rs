// SPDX-License-Identifier: MIT OR Apache-2.0

//! HII Configuration protocols.

use uefi_macros::unsafe_protocol;
use uefi_raw::protocol::hii::config::{ConfigKeywordHandlerProtocol, HiiConfigAccessProtocol};

/// The HII Keyword Handler Protocol.
///
/// # UEFI Spec Description
///
/// This protocol provides the mechanism to set and get the values associated
/// with a keyword exposed through a x-UEFI- prefixed configuration language namespace.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(ConfigKeywordHandlerProtocol::GUID)]
pub struct ConfigKeywordHandler(ConfigKeywordHandlerProtocol);

/// The HII Configuration Access Protocol.
///
/// # UEFI Spec Description
///
/// This protocol is responsible for facilitating access to configuration data from HII.
/// It is typically invoked by the HII Configuration Routing Protocol for handling
/// configuration requests. Forms browsers also interact with this protocol through
/// the `Callback()` function.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(HiiConfigAccessProtocol::GUID)]
pub struct HiiConfigAccess(HiiConfigAccessProtocol);
