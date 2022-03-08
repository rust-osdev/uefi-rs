//! `Rng` protocol.

use crate::{data_types::Guid, proto::Protocol, unsafe_guid, Result, Status};
use core::slice::SliceIndex;

/// Contain a Rng algorithm Guid
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RngAlgorithm(pub Guid);

impl RngAlgorithm {
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
        algorithm: Option<RngAlgorithm>,
        value_length: usize,
        value: *mut u8,
    ) -> Status,
}

impl Rng {
    pub fn get_info(&mut self, algorithm_list: &mut [RngAlgorithm]) -> Result<usize> {
        let mut algorithm_list_size = (algorithm_list.len() * 16) as *mut usize;

        (self.get_info)(self, algorithm_list_size, algorithm_list.as_mut_ptr())
            .into_with_val(|| algorithm_list_size as usize / 16)

        // TODO: Add AlgorithmType Enum for better visibility on algorithms
    }
}
