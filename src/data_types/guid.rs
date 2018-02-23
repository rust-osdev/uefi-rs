use core::fmt;

/// A globally unique identifier.
///
/// GUIDs are used by to identify protocols and other objects.
///
/// The difference from regular UUIDs is that the first 3 fields are
/// always encoded as little endian.
///
/// The `Display` formatter prints GUIDs in the UEFI-defined format:
/// `aabbccdd-eeff-gghh-iijj-kkllmmnnoopp`
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct Guid {
    /// The low field of the timestamp.
    a: u32,
    /// The middle field of the timestamp.
    b: u16,
    /// The high field of the timestamp multiplexed with the version number.
    c: u16,
    /// Contains:
    /// - The high field of the clock sequence multiplexed with the variant.
    /// - The low field of the clock sequence.
    /// - Spatially unique node identifier.
    d: [u8; 8],
}

impl Guid {
    /// Creates a new GUID from its component values.
    pub const fn from_values(a: u32, b: u16, c: u16, d: [u8; 8]) -> Self {
        Guid { a, b, c, d }
    }
}

impl fmt::Display for Guid {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let (a, b, c) = (self.a, self.b, self.c);

        let d = {
            let (low, high) = (self.d[0] as u16, self.d[1] as u16);

            (low << 8) | high
        };

        let e = {
            let node = &self.d[2..8];
            let mut e = 0;

            for i in 0..6 {
                e |= u64::from(node[5 - i]) << (i * 8);
            }

            e
        };

        write!(fmt, "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}", a, b, c, d, e)
    }
}
