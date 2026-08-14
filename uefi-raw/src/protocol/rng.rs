// SPDX-License-Identifier: MIT OR Apache-2.0

//! `Rng` protocol.

use crate::{Guid, Status, guid, newtype_enum};

newtype_enum! {
    /// Algorithm supported by a random-number generator.
    ///
    /// The defined algorithms are optional and not exhaustive. Vendors and
    /// future standards may define additional values.
    pub enum RngAlgorithmType: Guid => {
        /// Placeholder used to initialize an algorithm-list buffer.
        EMPTY_ALGORITHM = guid!("00000000-0000-0000-0000-000000000000"),

        /// Provides source entropy without a deterministic random-bit generator.
        ALGORITHM_RAW = guid!("e43176d7-b6e8-4827-b784-7ffdc4b68561"),

        /// NIST SP 800-90 Hash_DRBG algorithm identified by UEFI.
        ALGORITHM_SP800_90_HASH_256 = guid!("a7af67cb-603b-4d42-ba21-70bfb6293f96"),

        /// NIST SP 800-90 HMAC_DRBG algorithm identified by UEFI.
        ALGORITHM_SP800_90_HMAC_256 = guid!("c5149b43-ae85-4f53-9982-b94335d3a9e7"),

        /// NIST SP 800-90 CTR_DRBG algorithm identified by UEFI.
        ALGORITHM_SP800_90_CTR_256 = guid!("44f0de6e-4d8c-4045-a8c7-4dd168856b9e"),

        /// ANSI X9.31 generator using two-key or three-key 3DES.
        ALGORITHM_X9_31_3DES = guid!("63c4785a-ca34-4012-a3c8-0b6a324f5546"),

        /// ANSI X9.31 generator using AES.
        ALGORITHM_X9_31_AES = guid!("acd03321-777e-4d3d-b1c8-20cfd88820c9"),
    }
}

/// Random Number Generator protocol.
#[derive(Debug)]
#[repr(C)]
pub struct RngProtocol {
    pub get_info: unsafe extern "efiapi" fn(
        this: *mut Self,
        algorithm_list_size: *mut usize,
        algorithm_list: *mut RngAlgorithmType,
    ) -> Status,

    pub get_rng: unsafe extern "efiapi" fn(
        this: *mut Self,
        algorithm: *const RngAlgorithmType,
        value_length: usize,
        value: *mut u8,
    ) -> Status,
}

impl RngProtocol {
    pub const GUID: Guid = guid!("3152bca5-eade-433d-862e-c01cdc291f44");
}
