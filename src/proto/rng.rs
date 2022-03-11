//! `Rng` protocol.

use crate::{data_types::Guid, proto::Protocol, unsafe_guid, Result, Status};
use core::ptr;

/// Contain a Rng algorithm Guid
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RngAlgorithm(pub Guid);

impl RngAlgorithm {
    /// Get an empty `RngAlgorithm`
    ///
    /// Used provide a buffer to `Rng.get_info`
    pub fn default() -> Self {
        Self(Guid::default())
    }
}

/// Rng protocol
#[repr(C)]
#[unsafe_guid("3152bca5-eade-433d-862e-c01cdc291f44")]
#[derive(Protocol)]
pub struct Rng {
    get_info: extern "efiapi" fn(
        this: &Rng,
        algorithm_list_size: *mut usize,
        algorithm_list: *mut RngAlgorithm,
    ) -> Status,
    get_rng: extern "efiapi" fn(
        this: &Rng,
        algorithm: *const RngAlgorithm,
        value_length: usize,
        value: *mut u8,
    ) -> Status,
}

impl Rng {
    /// Returns information about the random number generation implementation.
    pub fn get_info(&mut self, algorithm_list: &mut [RngAlgorithm]) -> Result<usize> {
        let algorithm_list_size = (algorithm_list.len() * 16) as *mut usize;

        (self.get_info)(self, algorithm_list_size, algorithm_list.as_mut_ptr())
            .into_with_val(|| algorithm_list_size as usize / 16)

        // TODO: Add AlgorithmType Enum for better visibility on algorithms
    }

    /// Returns the next set of random numbers
    pub fn get_rng(&mut self, algorithm: Option<RngAlgorithm>, buffer: &mut [u8]) -> Result {
        let buffer_length = buffer.len();

        let algo = match algorithm {
            None => ptr::null(),
            Some(algo) => &algo as *const RngAlgorithm,
        };

        (self.get_rng)(self, algo, buffer_length, buffer.as_mut_ptr()).into()
    }
}
