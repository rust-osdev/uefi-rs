use uefi_sys::GUID;

/// A globally unique identifier
///
/// GUIDs are used by UEFI to identify protocols and other objects. They are
/// mostly like variant 2 UUIDs as specified by RFC 4122, but differ from them
/// in that the first 3 fields are little endian instead of big endian.
///
/// The `Display` formatter prints GUIDs in the canonical format defined by
/// RFC 4122, which is also used by UEFI.
pub type Guid = GUID;

/// Several entities in the UEFI specification can be referred to by their GUID,
/// this trait is a building block to interface them in uefi-rs.
///
/// You should never need to use the `Identify` trait directly, but instead go
/// for more specific traits such as `Protocol` or `FileProtocolInfo`, which
/// indicate in which circumstances an `Identify`-tagged type should be used.
///
/// Implementing `Identify` is unsafe because attaching an incorrect GUID to a
/// type can lead to type unsafety on both the Rust and UEFI side.
///
/// You can derive `Identify` for a type using the `unsafe_guid` procedural
/// macro, which is exported by this module. This macro mostly works like a
/// custom derive, but also supports type aliases. It takes a GUID in canonical
/// textual format as an argument, and is used in the following way:
///
/// ```
/// #[unsafe_guid("12345678-9abc-def0-1234-56789abcdef0")]
/// type Emptiness = ();
/// ```
pub unsafe trait Identify {
    /// Unique protocol identifier.
    const UNIQUE_GUID: Guid;
}

pub use uefi_macros::unsafe_guid;
