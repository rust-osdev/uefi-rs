use crate::arch::UefiArch;
use clap::{Parser, Subcommand};
use std::ops::Deref;
use std::path::PathBuf;

// Define some common options so that the doc strings don't have to be
// copy-pasted.

#[derive(Debug, Parser)]
pub struct TargetOpt {
    /// UEFI target to build for.
    #[clap(long, action, default_value_t)]
    pub target: UefiArch,
}

impl Deref for TargetOpt {
    type Target = UefiArch;

    fn deref(&self) -> &Self::Target {
        &self.target
    }
}

#[derive(Debug, Parser)]
pub struct ToolchainOpt {
    /// Rust toolchain to use, e.g. "nightly-2022-02-24".
    #[clap(long, action)]
    toolchain: Option<String>,
}

impl ToolchainOpt {
    /// Get the toolchain arg if set, otherwise use `default_toolchain`.
    pub fn or(&self, default_toolchain: &str) -> Option<String> {
        self.toolchain
            .clone()
            .or_else(|| Some(default_toolchain.to_string()))
    }
}

#[derive(Debug, Parser)]
pub struct BuildModeOpt {
    /// Build in release mode.
    #[clap(long, action)]
    pub release: bool,
}

#[derive(Debug, Parser)]
pub struct WarningOpt {
    /// Treat warnings as errors.
    #[clap(long, action)]
    pub warnings_as_errors: bool,
}

/// Developer utility for running various tasks in uefi-rs.
#[derive(Debug, Parser)]
pub struct Opt {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Debug, Subcommand)]
pub enum Action {
    Build(BuildOpt),
    Clippy(ClippyOpt),
    Doc(DocOpt),
    Miri(MiriOpt),
    Run(QemuOpt),
    Test(TestOpt),
    TestLatestRelease(TestLatestReleaseOpt),
}

/// Build all the uefi packages.
#[derive(Debug, Parser)]
pub struct BuildOpt {
    #[clap(flatten)]
    pub target: TargetOpt,

    #[clap(flatten)]
    pub toolchain: ToolchainOpt,

    #[clap(flatten)]
    pub build_mode: BuildModeOpt,
}

/// Run clippy on all the packages.
#[derive(Debug, Parser)]
pub struct ClippyOpt {
    #[clap(flatten)]
    pub target: TargetOpt,

    #[clap(flatten)]
    pub toolchain: ToolchainOpt,

    #[clap(flatten)]
    pub warning: WarningOpt,
}

/// Build the docs for the uefi packages.
#[derive(Debug, Parser)]
pub struct DocOpt {
    #[clap(flatten)]
    pub toolchain: ToolchainOpt,

    /// Open the docs in a browser.
    #[clap(long, action)]
    pub open: bool,

    #[clap(flatten)]
    pub warning: WarningOpt,
}

/// Run unit tests and doctests under Miri.
#[derive(Debug, Parser)]
pub struct MiriOpt {
    #[clap(flatten)]
    pub toolchain: ToolchainOpt,
}

/// Build uefi-test-runner and run it in QEMU.
#[derive(Debug, Parser)]
pub struct QemuOpt {
    #[clap(flatten)]
    pub target: TargetOpt,

    #[clap(flatten)]
    pub toolchain: ToolchainOpt,

    #[clap(flatten)]
    pub build_mode: BuildModeOpt,

    /// Disable hardware accelerated virtualization support in QEMU.
    #[clap(long, action)]
    pub disable_kvm: bool,

    /// Disable some tests that don't work in the CI.
    #[clap(long, action)]
    pub ci: bool,

    /// Run QEMU without a GUI.
    #[clap(long, action)]
    pub headless: bool,

    /// Directory in which to look for OVMF files.
    #[clap(long, action)]
    pub ovmf_dir: Option<PathBuf>,
}

/// Build uefi-test-runner and run it in QEMU.
#[derive(Debug, Parser)]
pub struct TestOpt;

/// Build the template against the crates.io packages.
#[derive(Debug, Parser)]
pub struct TestLatestReleaseOpt;
