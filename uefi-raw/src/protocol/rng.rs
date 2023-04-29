//! `Rng` protocol.

use crate::{guid, Guid, Status};

newtype_enum! {
    /// The algorithms listed are optional, not meant to be exhaustive
    /// and may be augmented by vendors or other industry standards.
    pub enum RngAlgorithmType: Guid => {
        /// Indicates a empty algorithm, used to instantiate a buffer
        /// for `get_info`
        EMPTY_ALGORITHM = guid!("00000000-0000-0000-0000-000000000000"),

        /// The “raw” algorithm, when supported, is intended to provide
        /// entropy directly from the source, without it going through
        /// some deterministic random bit generator.
        ALGORITHM_RAW = guid!("e43176d7-b6e8-4827-b784-7ffdc4b68561"),

        /// ALGORITHM_SP800_90_HASH_256
        ALGORITHM_SP800_90_HASH_256 = guid!("a7af67cb-603b-4d42-ba21-70bfb6293f96"),

        /// ALGORITHM_SP800_90_HMAC_256
        ALGORITHM_SP800_90_HMAC_256 = guid!("c5149b43-ae85-4f53-9982-b94335d3a9e7"),

        /// ALGORITHM_SP800_90_CTR_256
        ALGORITHM_SP800_90_CTR_256 = guid!("44f0de6e-4d8c-4045-a8c7-4dd168856b9e"),

        /// ALGORITHM_X9_31_3DES
        ALGORITHM_X9_31_3DES = guid!("63c4785a-ca34-4012-a3c8-0b6a324f5546"),

        /// ALGORITHM_X9_31_AES
        ALGORITHM_X9_31_AES = guid!("acd03321-777e-4d3d-b1c8-20cfd88820c9"),
    }
}

/// Rng protocol.
#[repr(C)]
pub struct RngProtocol {
    pub get_info: unsafe extern "efiapi" fn(
        this: *mut RngProtocol,
        algorithm_list_size: *mut usize,
        algorithm_list: *mut RngAlgorithmType,
    ) -> Status,

    pub get_rng: unsafe extern "efiapi" fn(
        this: *mut RngProtocol,
        algorithm: *const RngAlgorithmType,
        value_length: usize,
        value: *mut u8,
    ) -> Status,
}

impl RngProtocol {
    pub const GUID: Guid = guid!("3152bca5-eade-433d-862e-c01cdc291f44");
}
