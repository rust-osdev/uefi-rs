use core::fmt;

/// A revision of the UEFI specification.
///
/// The major revision number is incremented on major, API-incompatible changes.
///
/// The minor revision number is incremented on minor changes,
/// it is stored as a two-digit binary-coded decimal.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Revision(u32);

// Allow missing docs, there's nothing useful to document about these
// constants.
#[allow(missing_docs)]
impl Revision {
    pub const EFI_1_02: Self = Self::new(1, 2);
    pub const EFI_1_10: Self = Self::new(1, 10);
    pub const EFI_2_00: Self = Self::new(2, 00);
    pub const EFI_2_10: Self = Self::new(2, 10);
    pub const EFI_2_20: Self = Self::new(2, 20);
    pub const EFI_2_30: Self = Self::new(2, 30);
    pub const EFI_2_31: Self = Self::new(2, 31);
    pub const EFI_2_40: Self = Self::new(2, 40);
    pub const EFI_2_50: Self = Self::new(2, 50);
    pub const EFI_2_60: Self = Self::new(2, 60);
    pub const EFI_2_70: Self = Self::new(2, 70);
    pub const EFI_2_80: Self = Self::new(2, 80);
    pub const EFI_2_90: Self = Self::new(2, 90);
}

impl Revision {
    /// Creates a new revision.
    pub const fn new(major: u16, minor: u16) -> Self {
        let major = major as u32;
        let minor = minor as u32;
        let value = (major << 16) | minor;
        Revision(value)
    }

    /// Returns the major revision.
    pub const fn major(self) -> u16 {
        (self.0 >> 16) as u16
    }

    /// Returns the minor revision.
    pub const fn minor(self) -> u16 {
        self.0 as u16
    }
}

impl fmt::Debug for Revision {
    /// Formats the revision in the `major.minor.patch` format.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (major, minor) = (self.major(), self.minor());
        write!(f, "{}.{}.{}", major, minor / 10, minor % 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_revision() {
        let rev = Revision::EFI_2_31;
        assert_eq!(rev.major(), 2);
        assert_eq!(rev.minor(), 31);
        assert_eq!(rev.0, 0x0002_001f);

        assert!(Revision::EFI_1_10 < Revision::EFI_2_00);
    }
}
