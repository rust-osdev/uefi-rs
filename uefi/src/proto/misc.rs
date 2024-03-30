//! Miscellaneous protocols.

use crate::proto::unsafe_protocol;
use crate::{Result, StatusExt};
use uefi_raw::protocol::misc::{TimestampProperties, TimestampProtocol};

/// Protocol for retrieving a high-resolution timestamp counter.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(TimestampProtocol::GUID)]
pub struct Timestamp(TimestampProtocol);

impl Timestamp {
    /// Get the current value of the timestamp counter.
    #[must_use]
    pub fn get_timestamp(&self) -> u64 {
        unsafe { (self.0.get_timestamp)() }
    }

    /// Get the properties of the timestamp counter.
    pub fn get_properties(&self) -> Result<TimestampProperties> {
        let mut properties = TimestampProperties::default();
        unsafe { (self.0.get_properties)(&mut properties) }.to_result_with_val(|| properties)
    }
}
