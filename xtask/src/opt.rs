use crate::arch::UefiArch;
use clap::{Parser, Subcommand};
use std::ops::Deref;
use std::path::PathBuf;

// Define some common options so that the doc strings don't have to be
// copy-pasted.

#[derive(Debug, Parser)]
pub struct TargetOpt {
    /// UEFI target to build for.
    #[clap(long, default_value_t)]
    pub target: UefiArch,
}

impl Deref for TargetOpt {
    type Target = UefiArch;

    fn deref(&self) -> &Self::Target {
        &self.target
    }
}

#[derive(Debug, Parser)]
pub struct BuildModeOpt {
    /// Build in release mode.
    #[clap(long)]
    pub release: bool,
}

#[derive(Debug, Parser)]
pub struct WarningOpt {
    /// Treat warnings as errors.
    #[clap(long)]
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
    pub build_mode: BuildModeOpt,
}

/// Run clippy on all the packages.
#[derive(Debug, Parser)]
pub struct ClippyOpt {
    #[clap(flatten)]
    pub target: TargetOpt,

    #[clap(flatten)]
    pub warning: WarningOpt,
}

/// Build the docs for the uefi packages.
#[derive(Debug, Parser)]
pub struct DocOpt {
    /// Open the docs in a browser.
    #[clap(long)]
    pub open: bool,

    #[clap(flatten)]
    pub warning: WarningOpt,
}

/// Build uefi-test-runner and run it in QEMU.
#[derive(Debug, Parser)]
pub struct QemuOpt {
    #[clap(flatten)]
    pub target: TargetOpt,

    #[clap(flatten)]
    pub build_mode: BuildModeOpt,

    /// Disable hardware accelerated virtualization support in QEMU.
    #[clap(long)]
    pub disable_kvm: bool,

    /// Disable some tests that don't work in the CI.
    #[clap(long)]
    pub ci: bool,

    /// Run QEMU without a GUI.
    #[clap(long)]
    pub headless: bool,

    /// Directory in which to look for OVMF files.
    #[clap(long)]
    pub ovmf_dir: Option<PathBuf>,
}

/// Build uefi-test-runner and run it in QEMU.
#[derive(Debug, Parser)]
pub struct TestOpt;

/// Build the template against the crates.io packages.
#[derive(Debug, Parser)]
pub struct TestLatestReleaseOpt;
