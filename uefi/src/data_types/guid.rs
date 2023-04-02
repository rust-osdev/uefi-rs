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
    #[must_use]
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

    /// Create a GUID from a 16-byte array. No changes to byte order are made.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        let a = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let b = u16::from_le_bytes([bytes[4], bytes[5]]);
        let c = u16::from_le_bytes([bytes[6], bytes[7]]);
        let d = [
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ];

        Self { a, b, c, d }
    }

    /// Convert to a 16-byte array.
    #[must_use]
    #[rustfmt::skip]
    pub const fn to_bytes(self) -> [u8; 16] {
        let a = self.a.to_le_bytes();
        let b = self.b.to_le_bytes();
        let c = self.c.to_le_bytes();
        let d = self.d;

        [
            a[0], a[1], a[2], a[3],
            b[0], b[1], c[0], c[1],
            d[0], d[1], d[2], d[3],
            d[4], d[5], d[6], d[7],
        ]
    }
}

impl fmt::Display for Guid {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let a = self.a;
        let b = self.b;
        let c = self.c;

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

        write!(fmt, "{a:08x}-{b:04x}-{c:04x}-{d:04x}-{e:012x}",)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guid;

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
    fn test_guid_macro() {
        assert_eq!(
            guid!("12345678-9abc-def0-1234-56789abcdef0"),
            Guid::from_values(0x12345678, 0x9abc, 0xdef0, 0x1234, 0x56789abcdef0)
        );
    }

    #[test]
    fn test_to_from_bytes() {
        #[rustfmt::skip]
        let bytes = [
            0x78, 0x56, 0x34, 0x12,
            0xbc, 0x9a,
            0xf0, 0xde,
            0x12, 0x34,
            0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
        ];
        assert_eq!(
            Guid::from_bytes(bytes),
            Guid::from_values(0x12345678, 0x9abc, 0xdef0, 0x1234, 0x56789abcdef0)
        );
        assert_eq!(Guid::from_bytes(bytes).to_bytes(), bytes);
    }
}
