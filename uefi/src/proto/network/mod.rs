// SPDX-License-Identifier: MIT OR Apache-2.0

//! Network access protocols.
//!
//! These protocols can be used to interact with network resources.
//!
//! All high-level wrappers will accept [`core::net`] types:
//! - [`IpAddr`]
//! - [`Ipv4Addr`]
//! - [`Ipv6Addr`]
//!
//! The only exception is [`uefi_raw::MacAddress`] which doesn't have a
//! corresponding type in the standard library.
//!
//! [`IpAddr`]: core::net::IpAddr
//! [`Ipv4Addr`]: core::net::Ipv4Addr
//! [`Ipv6Addr`]: core::net::Ipv6Addr

pub mod http;
pub mod ip4config2;
pub mod pxe;
pub mod snp;
