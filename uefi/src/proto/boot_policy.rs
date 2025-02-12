// SPDX-License-Identifier: MIT OR Apache-2.0

//! Module for the [`BootPolicy`] helper type.

use uefi_raw::Boolean;

/// The UEFI boot policy is a property that influences the behaviour of
/// various UEFI functions that load files (typically UEFI images).
///
/// This type is not ABI compatible. On the ABI level, this corresponds to
/// a [`Boolean`].
#[derive(Copy, Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum BootPolicy {
    /// Indicates that the request originates from the boot manager, and that
    /// the boot manager is attempting to load the provided `file_path` as a
    /// boot selection.
    ///
    /// Boot selection refers to what a user has chosen in the (GUI) boot menu.
    ///
    /// This corresponds to the underlying [`Boolean`] being `true`.
    BootSelection,
    /// The provided `file_path` must match an exact file to be loaded.
    ///
    /// This corresponds to the underlying [`Boolean`] being `false`.
    #[default]
    ExactMatch,
}

impl From<BootPolicy> for Boolean {
    fn from(value: BootPolicy) -> Self {
        match value {
            BootPolicy::BootSelection => Self::TRUE,
            BootPolicy::ExactMatch => Self::FALSE,
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
        assert_eq!(
            BootPolicy::try_from(Boolean::TRUE).unwrap(),
            BootPolicy::BootSelection
        );
        assert_eq!(
            BootPolicy::try_from(Boolean::FALSE).unwrap(),
            BootPolicy::ExactMatch
        );
        assert_eq!(Boolean::from(BootPolicy::BootSelection), Boolean::TRUE);
        assert_eq!(Boolean::from(BootPolicy::ExactMatch), Boolean::FALSE);
    }
}
