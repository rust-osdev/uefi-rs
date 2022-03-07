//! `Rng` protocol.

use crate::{data_types::Guid, proto::Protocol, unsafe_guid, Status};

/// Contain a Rng algorithm Guid
#[repr(C)]
struct RngAlgorithm(Guid);

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

/*impl Rng {
    fn get_info(&mut self) -> Result<[RngAlgorithm]> {
        let mut algorithm_list_size: usize;
        let mut algorithm_list: []
    }
}*/
