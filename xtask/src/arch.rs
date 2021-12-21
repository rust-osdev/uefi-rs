use anyhow::{anyhow, Error, Result};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UefiArch {
    AArch64,
    X86_64,
}

impl UefiArch {
    fn all() -> &'static [Self] {
        &[Self::AArch64, Self::X86_64]
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::AArch64 => "aarch64",
            Self::X86_64 => "x86_64",
        }
    }

    pub fn as_triple(self) -> String {
        format!("{}-unknown-uefi", self.as_str())
    }
}

impl Default for UefiArch {
    fn default() -> Self {
        Self::X86_64
    }
}

impl fmt::Display for UefiArch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for UefiArch {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::all()
            .iter()
            .find(|arch| arch.as_str() == s)
            .cloned()
            .ok_or_else(|| anyhow!("invalid triple: {}", s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(UefiArch::from_str("x86_64").unwrap(), UefiArch::X86_64);
    }
}
