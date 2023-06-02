//! `Rng` protocol.

use crate::proto::unsafe_protocol;
use crate::{Result, Status, StatusExt};
use core::{mem, ptr};

pub use uefi_raw::protocol::rng::RngAlgorithmType;

/// Rng protocol
#[repr(transparent)]
#[unsafe_protocol(uefi_raw::protocol::rng::RngProtocol::GUID)]
pub struct Rng(uefi_raw::protocol::rng::RngProtocol);

impl Rng {
    /// Returns information about the random number generation implementation.
    pub fn get_info<'buf>(
        &mut self,
        algorithm_list: &'buf mut [RngAlgorithmType],
    ) -> Result<&'buf [RngAlgorithmType], Option<usize>> {
        let mut algorithm_list_size = mem::size_of_val(algorithm_list);

        unsafe {
            (self.0.get_info)(
                &mut self.0,
                &mut algorithm_list_size,
                algorithm_list.as_mut_ptr(),
            )
            .to_result_with(
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

        unsafe {
            (self.0.get_rng)(&mut self.0, algo, buffer_length, buffer.as_mut_ptr()).to_result()
        }
    }
}
