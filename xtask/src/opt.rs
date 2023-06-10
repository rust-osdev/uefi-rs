use crate::arch::UefiArch;
use clap::{Parser, Subcommand, ValueEnum};
use std::ops::Deref;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum TpmVersion {
    V1,
    V2,
}

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
pub struct BuildModeOpt {
    /// Build in release mode.
    #[clap(long, action)]
    pub release: bool,
}

#[derive(Debug, Parser)]
pub struct UnstableOpt {
    /// Enable the `unstable` feature (requires nightly).
    #[clap(long, action)]
    pub unstable: bool,
}

impl Deref for UnstableOpt {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.unstable
    }
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
    CheckRaw(CheckRawOpt),
    Clippy(ClippyOpt),
    Doc(DocOpt),
    GenCode(GenCodeOpt),
    Miri(MiriOpt),
    Run(QemuOpt),
    Test(TestOpt),
    Fmt(FmtOpt),
}

/// Build all the uefi packages.
#[derive(Debug, Parser)]
pub struct BuildOpt {
    #[clap(flatten)]
    pub target: TargetOpt,

    #[clap(flatten)]
    pub build_mode: BuildModeOpt,

    /// Build multiple times to check that different feature
    /// combinations work.
    #[clap(long, action)]
    pub feature_permutations: bool,
}

/// Check various properties of the uefi-raw code.
#[derive(Debug, Parser)]
pub struct CheckRawOpt {}

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
    #[clap(long, action)]
    pub open: bool,

    /// Tells whether private items should be documented. This is convenient to check for
    /// broken intra-doc links in private items.
    #[clap(long, action)]
    pub document_private_items: bool,

    #[clap(flatten)]
    pub unstable: UnstableOpt,

    #[clap(flatten)]
    pub warning: WarningOpt,
}

/// Update auto-generated Rust code.
///
/// This is used to generate `device_path_gen.rs`. See
/// `xtask/src/device_path/README.md` for more information.
#[derive(Debug, Parser)]
pub struct GenCodeOpt {
    /// Exit 0 if the generated code is up-to-date, exit 1 if not.
    #[clap(long)]
    pub check: bool,
}

/// Run unit tests and doctests under Miri.
#[derive(Debug, Parser)]
pub struct MiriOpt {}

/// Build uefi-test-runner and run it in QEMU.
#[derive(Debug, Parser)]
pub struct QemuOpt {
    #[clap(flatten)]
    pub target: TargetOpt,

    #[clap(flatten)]
    pub build_mode: BuildModeOpt,

    /// Disable hardware accelerated virtualization support in QEMU.
    #[clap(long, action)]
    pub disable_kvm: bool,

    /// Disable network tests.
    #[clap(long, action)]
    pub disable_network: bool,

    /// Disable some tests that don't work in the CI.
    #[clap(long, action)]
    pub ci: bool,

    /// Run QEMU without a GUI.
    #[clap(long, action)]
    pub headless: bool,

    /// Attach a software TPM to QEMU.
    #[clap(long, action)]
    pub tpm: Option<TpmVersion>,

    /// Path of an OVMF code file.
    #[clap(long, action)]
    pub ovmf_code: Option<PathBuf>,

    /// Path of an OVMF vars file.
    #[clap(long, action)]
    pub ovmf_vars: Option<PathBuf>,

    /// Run an example instead of the main binary.
    #[clap(long, action)]
    pub example: Option<String>,

    #[clap(flatten)]
    pub unstable: UnstableOpt,
}

/// Run unit tests and doctests on the host.
#[derive(Debug, Parser)]
pub struct TestOpt {
    #[clap(flatten)]
    pub unstable: UnstableOpt,

    /// Skip the uefi-macros tests.
    #[clap(long, action)]
    pub skip_macro_tests: bool,
}

/// Run formatting on the repo.
#[derive(Debug, Parser)]
pub struct FmtOpt {
    /// Just check but do not write files.
    #[clap(long, action)]
    pub check: bool,
}
