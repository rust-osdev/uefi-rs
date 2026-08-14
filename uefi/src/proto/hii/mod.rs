// SPDX-License-Identifier: MIT OR Apache-2.0

//! UEFI HII protocols and data types.

pub mod config;
#[cfg(feature = "alloc")]
pub mod config_routing;
#[cfg(feature = "alloc")]
pub mod config_str;
#[cfg(feature = "alloc")]
pub mod database;
