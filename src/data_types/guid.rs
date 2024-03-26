use core::fmt;

/// A globally unique identifier
///
/// GUIDs are used by UEFI to identify protocols and other objects. They are
/// mostly like variant 2 UUIDs as specified by RFC 4122, but differ from them
/// in that the first 3 fields are little endian instead of big endian.
///
/// The `Display` formatter prints GUIDs in the canonical format defined by
/// RFC 4122, which is also used by UEFI.
#[derive(Debug, Default, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Guid {
    /// The low field of the timestamp.
    a: u32,
    /// The middle field of the timestamp.
    b: u16,
    /// The high field of the timestamp multiplexed with the version number.
    c: u16,
    /// Contains, in this order:
    /// - The high field of the clock sequence multiplexed with the variant.
    /// - The low field of the clock sequence.
    /// - The spatially unique node identifier.
    d: [u8; 8],
}

impl Guid {
    /// Creates a new GUID from its canonical representation
    pub const fn from_values(
        time_low: u32,
        time_mid: u16,
        time_high_and_version: u16,
        clock_seq_and_variant: u16,
        node: u64,
    ) -> Self {
        assert!(node.leading_zeros() >= 16, "node must be a 48-bit integer");
        // intentional shadowing
        let node = node.to_be_bytes();

        Guid {
            a: time_low,
            b: time_mid,
            c: time_high_and_version,
            d: [
                (clock_seq_and_variant / 0x100) as u8,
                (clock_seq_and_variant % 0x100) as u8,
                // first two elements of node are ignored, we only want the low 48 bits
                node[2],
                node[3],
                node[4],
                node[5],
                node[6],
                node[7],
            ],
        }
    }
}

impl fmt::Display for Guid {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let d = {
            let mut buf = [0u8; 2];
            buf[..].copy_from_slice(&self.d[0..2]);
            u16::from_be_bytes(buf)
        };

        let e = {
            let mut buf = [0u8; 8];
            // first two elements of node are ignored, we only want the low 48 bits
            buf[2..].copy_from_slice(&self.d[2..8]);
            u64::from_be_bytes(buf)
        };

        write!(
            fmt,
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            self.a, self.b, self.c, d, e
        )
    }
}

/// Several entities in the UEFI specification can be referred to by their GUID,
/// this trait is a building block to interface them in uefi-rs.
///
/// You should never need to use the `Identify` trait directly, but instead go
/// for more specific traits such as `Protocol` or `FileProtocolInfo`, which
/// indicate in which circumstances an `Identify`-tagged type should be used.
///
/// # Safety
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
/// use uefi::unsafe_guid;
/// #[unsafe_guid("12345678-9abc-def0-1234-56789abcdef0")]
/// struct Emptiness;
/// ```
pub unsafe trait Identify {
    /// Unique protocol identifier.
    const GUID: Guid;
}

pub use uefi_macros::unsafe_guid;

#[cfg(test)]
mod tests {
    use uefi::unsafe_guid;
    extern crate alloc;
    use super::*;

    #[test]
    fn test_guid_display() {
        assert_eq!(
            alloc::format!(
                "{}",
                Guid::from_values(0x12345678, 0x9abc, 0xdef0, 0x1234, 0x56789abcdef0)
            ),
            "12345678-9abc-def0-1234-56789abcdef0"
        );
    }

    #[test]
    fn test_unsafe_guid() {
        #[unsafe_guid("12345678-9abc-def0-1234-56789abcdef0")]
        struct X;

        assert_eq!(
            X::GUID,
            Guid::from_values(0x12345678, 0x9abc, 0xdef0, 0x1234, 0x56789abcdef0)
        );
    }
}
