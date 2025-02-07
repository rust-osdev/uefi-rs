// SPDX-License-Identifier: MIT OR Apache-2.0

use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, clap::ValueEnum)]
pub enum UefiArch {
    #[value(name = "aarch64")]
    AArch64,

    #[value(name = "ia32")]
    IA32,

    #[value(name = "x86_64")]
    X86_64,
}

impl UefiArch {
    fn as_str(self) -> &'static str {
        match self {
            Self::AArch64 => "aarch64",
            Self::IA32 => "ia32",
            Self::X86_64 => "x86_64",
        }
    }

    pub fn as_triple(self) -> &'static str {
        match self {
            Self::AArch64 => "aarch64-unknown-uefi",
            Self::IA32 => "i686-unknown-uefi",
            Self::X86_64 => "x86_64-unknown-uefi",
        }
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
