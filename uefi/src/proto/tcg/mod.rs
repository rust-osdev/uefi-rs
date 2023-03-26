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

use bitflags::bitflags;

/// Platform Configuration Register (PCR) index.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct PcrIndex(pub u32);

bitflags! {
    /// Hash algorithms the protocol can provide.
    ///
    /// The [`v1`] protocol only supports SHA1.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(transparent)]
    pub struct HashAlgorithm: u32 {
        /// SHA-1 hash.
        const SHA1 = 0x0000_0001;

        /// SHA-256 hash.
        const SHA256 = 0x0000_0002;

        /// SHA-384 hash.
        const SHA384 = 0x0000_0004;

        /// SHA-512 hash.
        const SHA512 = 0x0000_0008;

        /// SM3-256 hash.
        const SM3_256 = 0x0000_0010;
    }
}
