//! Protocol definitions.
//!
//! Protocols are sets of related functionality.
//!
//! Protocols are identified by a unique ID.
//!
//! Protocols can be implemented by a UEFI driver,
//! and are usually retrieved from a standard UEFI table or
//! by querying a handle.

use crate::Identify;

/// Common trait implemented by all standard UEFI protocols
///
/// According to the UEFI's specification, protocols are !Send (they expect to
/// be run on the bootstrap processor) and !Sync (they are not thread-safe).
/// You can derive the Protocol trait, add these bounds, and specify the
/// Protocol's GUID using the following syntax:
///
/// ```
/// #[derive(Identify, Protocol)]
/// #[unsafe_guid(0x1234_5678, 0x9abc, 0xdef0, 0x1234, 0x5678_9abc_def0)]
/// struct DummyProtocol {}
/// ```
pub trait Protocol: Identify {}

pub mod console;
pub mod debug;
pub mod media;
