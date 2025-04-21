// SPDX-License-Identifier: MIT OR Apache-2.0

//! Network access protocols.
//!
//! These protocols can be used to interact with network resources.
//!
//! To work with Mac and IP addresses, `uefi` uses with the types:
//! - [`EfiIpAddr`] that is tightly integrated with the [`core::net::IpAddr`]
//!   type,
//! - [`EfiMacAddr`]

pub mod http;
pub mod ip4config2;
pub mod pxe;
pub mod snp;

pub use uefi_raw::{IpAddress as EfiIpAddr, MacAddress as EfiMacAddr};
