pub use uguid::Guid;

/// Several entities in the UEFI specification can be referred to by their GUID,
/// this trait is a building block to interface them in uefi-rs.
///
/// You should never need to use the `Identify` trait directly, but instead go
/// for more specific traits such as [`Protocol`] or [`FileProtocolInfo`], which
/// indicate in which circumstances an `Identify`-tagged type should be used.
///
/// For the common case of implementing this trait for a protocol, use
/// the [`unsafe_protocol`] macro.
///
/// # Safety
///
/// Implementing `Identify` is unsafe because attaching an incorrect GUID to a
/// type can lead to type unsafety on both the Rust and UEFI side.
///
/// [`Protocol`]: crate::proto::Protocol
/// [`FileProtocolInfo`]: crate::proto::media::file::FileProtocolInfo
/// [`unsafe_protocol`]: crate::proto::unsafe_protocol
pub unsafe trait Identify {
    /// Unique protocol identifier.
    const GUID: Guid;
}
