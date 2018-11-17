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
        let d = {
            let (low, high) = (u16::from(self.d[0]), u16::from(self.d[1]));

            (low << 8) | high
        };

        // Extract and reverse byte order.
        let e = self.d[2..8].iter().enumerate().fold(0, |acc, (i, &elem)| {
            acc | {
                let shift = (5 - i) * 8;
                u64::from(elem) << shift
            }
        });

        write!(
            fmt,
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            self.a, self.b, self.c, d, e
        )
    }
}

/// Several entities in the UEFI specification can be referred to by their GUID,
/// this trait is a building block to interface this in uefi-rs.
pub trait Identify {
    /// Unique protocol identifier.
    const GUID: Guid;
}
