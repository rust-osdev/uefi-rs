//! `Rng` protocol.

use crate::{data_types::Guid, guid, proto::Protocol, unsafe_guid, Result, Status};
use core::{mem, ptr};

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

/// Rng protocol
#[repr(C)]
#[unsafe_guid("3152bca5-eade-433d-862e-c01cdc291f44")]
#[derive(Protocol)]
pub struct Rng {
    get_info: unsafe extern "efiapi" fn(
        this: &Rng,
        algorithm_list_size: *mut usize,
        algorithm_list: *mut RngAlgorithmType,
    ) -> Status,
    get_rng: unsafe extern "efiapi" fn(
        this: &Rng,
        algorithm: *const RngAlgorithmType,
        value_length: usize,
        value: *mut u8,
    ) -> Status,
}

impl Rng {
    /// Returns information about the random number generation implementation.
    pub fn get_info<'buf>(
        &mut self,
        algorithm_list: &'buf mut [RngAlgorithmType],
    ) -> Result<&'buf [RngAlgorithmType], Option<usize>> {
        let mut algorithm_list_size = algorithm_list.len() * mem::size_of::<RngAlgorithmType>();

        unsafe {
            (self.get_info)(self, &mut algorithm_list_size, algorithm_list.as_mut_ptr()).into_with(
                || {
                    let len = algorithm_list_size / mem::size_of::<RngAlgorithmType>();
                    &algorithm_list[..len]
                },
                |status| {
                    if status == Status::BUFFER_TOO_SMALL {
                        Some(algorithm_list_size)
                    } else {
                        None
                    }
                },
            )
        }
    }

    /// Returns the next set of random numbers
    pub fn get_rng(&mut self, algorithm: Option<RngAlgorithmType>, buffer: &mut [u8]) -> Result {
        let buffer_length = buffer.len();

        let algo = match algorithm.as_ref() {
            None => ptr::null(),
            Some(algo) => algo as *const RngAlgorithmType,
        };

        unsafe { (self.get_rng)(self, algo, buffer_length, buffer.as_mut_ptr()).into() }
    }
}
