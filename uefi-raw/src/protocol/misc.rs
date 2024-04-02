use crate::{guid, Guid, Status};

#[derive(Debug)]
#[repr(C)]
pub struct TimestampProtocol {
    pub get_timestamp: unsafe extern "efiapi" fn() -> u64,
    pub get_properties: unsafe extern "efiapi" fn(*mut TimestampProperties) -> Status,
}

impl TimestampProtocol {
    pub const GUID: Guid = guid!("afbfde41-2e6e-4262-ba65-62b9236e5495");
}

/// Properties of the timestamp counter.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(C)]
pub struct TimestampProperties {
    /// Timestamp counter frequency, in Hz.
    pub frequency: u64,

    /// The maximum value of the timestamp counter before it rolls over. For
    /// example, a 24-bit counter would have an end value of `0xff_ffff`.
    pub end_value: u64,
}
