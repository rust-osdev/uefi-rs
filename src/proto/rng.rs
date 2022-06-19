//! `Rng` protocol.

use crate::{data_types::Guid, proto::Protocol, unsafe_guid, Result, Status};
use core::{mem, ptr};

newtype_enum! {
    /// The algorithms listed are optional, not meant to be exhaustive
    /// and may be augmented by vendors or other industry standards.
    pub enum RngAlgorithmType: Guid => {
        /// Indicates a empty algorithm, used to instantiate a buffer
        /// for `get_info`
        EMPTY_ALGORITHM = Guid::from_values(
            0x00000000,
            0x0000,
            0x0000,
            0x0000,
            0x000000000000,
        ),

        /// The “raw” algorithm, when supported, is intended to provide
        /// entropy directly from the source, without it going through
        /// some deterministic random bit generator.
        ALGORITHM_RAW = Guid::from_values(
            0xe43176d7,
            0xb6e8,
            0x4827,
            0xb784,
            0x7ffdc4b68561,
        ),

        /// ALGORITHM_SP800_90_HASH_256
        ALGORITHM_SP800_90_HASH_256 = Guid::from_values(
            0xa7af67cb,
            0x603b,
            0x4d42,
            0xba21,
            0x70bfb6293f96,
        ),

        /// ALGORITHM_SP800_90_HMAC_256
        ALGORITHM_SP800_90_HMAC_256 = Guid::from_values(
            0xc5149b43,
            0xae85,
            0x4f53,
            0x9982,
            0xb94335d3a9e7,
        ),

        /// ALGORITHM_SP800_90_CTR_256
        ALGORITHM_SP800_90_CTR_256 = Guid::from_values(
            0x44f0de6e,
            0x4d8c,
            0x4045,
            0xa8c7,
            0x4dd168856b9e,
        ),

        /// ALGORITHM_X9_31_3DES
        ALGORITHM_X9_31_3DES = Guid::from_values(
            0x63c4785a,
            0xca34,
            0x4012,
            0xa3c8,
            0x0b6a324f5546,
        ),

        /// ALGORITHM_X9_31_AES
        ALGORITHM_X9_31_AES = Guid::from_values(
            0xacd03321,
            0x777e,
            0x4d3d,
            0xb1c8,
            0x20cfd88820c9,
        ),
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
