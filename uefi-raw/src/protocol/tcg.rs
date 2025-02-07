// SPDX-License-Identifier: MIT OR Apache-2.0

//! [TCG] (Trusted Computing Group) protocols.
//!
//! These protocols provide access to the [TPM][tpm] (Trusted Platform Module).
//!
//! There are two versions of the protocol. The original protocol is in
//! the [`v1`] module. It is used with TPM 1.1 and 1.2 devices. The
//! newer protocol in the [`v2`] module is generally provided for TPM
//! 2.0 devices, although the spec indicates it can be used for older
//! TPM versions as well.
//!
//! [TCG]: https://trustedcomputinggroup.org/
//! [TPM]: https://en.wikipedia.org/wiki/Trusted_Platform_Module

pub mod v1;
pub mod v2;

mod enums;
pub use enums::*;
