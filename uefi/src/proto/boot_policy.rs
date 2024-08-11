// SPDX-License-Identifier: MIT OR Apache-2.0

//! Module for the [`BootPolicy`] helper type.

use uefi_raw::Boolean;

/// The UEFI boot policy is a property that influences the behaviour of
/// various UEFI functions that load files (typically UEFI images).
///
/// This type is not ABI compatible. On the ABI level, this is an UEFI
/// boolean.
#[derive(Copy, Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum BootPolicy {
    /// Indicates that the request originates from the boot manager, and that
    /// the boot manager is attempting to load the provided `file_path` as a
    /// boot selection.
    ///
    /// Boot selection refers to what a user has chosen in the (GUI) boot menu.
    ///
    /// This corresponds to the `TRUE` value in the UEFI spec.
    BootSelection,
    /// The provided `file_path` must match an exact file to be loaded.
    ///
    /// This corresponds to the `FALSE` value in the UEFI spec.
    #[default]
    ExactMatch,
}

impl From<BootPolicy> for bool {
    fn from(value: BootPolicy) -> Self {
        match value {
            BootPolicy::BootSelection => Boolean::TRUE,
            BootPolicy::ExactMatch => Boolean::FALSE,
        }
    }
}

impl From<Boolean> for BootPolicy {
    fn from(value: Boolean) -> Self {
        let boolean: bool = value.into();
        match boolean {
            true => Self::BootSelection,
            false => Self::ExactMatch,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_policy() {
        assert_eq!(bool::from(BootPolicy::ExactMatch), false);
        assert_eq!(bool::from(BootPolicy::BootSelection), true);

        assert_eq!(BootPolicy::from(false), BootPolicy::ExactMatch);
        assert_eq!(BootPolicy::from(true), BootPolicy::BootSelection);

        assert_eq!(u8::from(BootPolicy::ExactMatch), 0);
        assert_eq!(u8::from(BootPolicy::BootSelection), 1);

        assert_eq!(BootPolicy::try_from(0), Ok(BootPolicy::ExactMatch));
        assert_eq!(BootPolicy::try_from(1), Ok(BootPolicy::BootSelection));
        assert_eq!(
            BootPolicy::try_from(2),
            Err(BootPolicyError::InvalidInteger(2))
        );
    }
}
