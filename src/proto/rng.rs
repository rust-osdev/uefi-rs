//! `Rng` protocol.

use crate::{data_types::Guid, proto::Protocol, unsafe_guid, Result, Status};
use core::{mem, ptr};

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
    get_info: unsafe extern "efiapi" fn(
        this: &Rng,
        algorithm_list_size: *mut usize,
        algorithm_list: *mut RngAlgorithm,
    ) -> Status,
    get_rng: unsafe extern "efiapi" fn(
        this: &Rng,
        algorithm: *const RngAlgorithm,
        value_length: usize,
        value: *mut u8,
    ) -> Status,
}

impl Rng {
    /// Returns information about the random number generation implementation.
    pub fn get_info<'buf>(
        &mut self,
        algorithm_list: &'buf mut [RngAlgorithm],
    ) -> Result<&'buf [RngAlgorithm], Option<usize>> {
        let mut algorithm_list_size = algorithm_list.len() * mem::size_of::<RngAlgorithm>();

        unsafe {
            (self.get_info)(self, &mut algorithm_list_size, algorithm_list.as_mut_ptr()).into_with(
                || {
                    let len = algorithm_list_size / mem::size_of::<RngAlgorithm>();
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
    pub fn get_rng(&mut self, algorithm: Option<RngAlgorithm>, buffer: &mut [u8]) -> Result {
        let buffer_length = buffer.len();

        let algo = match algorithm {
            None => ptr::null(),
            Some(algo) => &algo as *const RngAlgorithm,
        };

        unsafe { (self.get_rng)(self, algo, buffer_length, buffer.as_mut_ptr()).into() }
    }
}
