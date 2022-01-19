mod arch;
mod cargo;
mod opt;
mod qemu;
mod util;

use anyhow::Result;
use cargo::{Cargo, CargoAction, Feature, Package};
use clap::Parser;
use opt::{Action, BuildOpt, ClippyOpt, DocOpt, Opt, QemuOpt};
use util::run_cmd;

fn build(opt: &BuildOpt) -> Result<()> {
    let cargo = Cargo {
        action: CargoAction::Build,
        features: Feature::more_code(),
        nightly: true,
        packages: Package::all_except_xtask(),
        release: opt.build_mode.release,
        target: Some(*opt.target),
        warnings_as_errors: false,
    };
    run_cmd(cargo.command()?)
}

fn clippy(opt: &ClippyOpt) -> Result<()> {
    // Run clippy on all the UEFI packages.
    let cargo = Cargo {
        action: CargoAction::Clippy,
        features: Feature::more_code(),
        nightly: true,
        packages: Package::all_except_xtask(),
        release: false,
        target: Some(*opt.target),
        warnings_as_errors: opt.warning.warnings_as_errors,
    };
    run_cmd(cargo.command()?)?;

    // Run clippy on xtask.
    let cargo = Cargo {
        action: CargoAction::Clippy,
        features: Vec::new(),
        nightly: false,
        packages: vec![Package::Xtask],
        release: false,
        target: None,
        warnings_as_errors: opt.warning.warnings_as_errors,
    };
    run_cmd(cargo.command()?)
}

/// Build docs.
fn doc(opt: &DocOpt) -> Result<()> {
    let cargo = Cargo {
        action: CargoAction::Doc { open: opt.open },
        features: Feature::more_code(),
        nightly: true,
        packages: Package::published(),
        release: false,
        target: None,
        warnings_as_errors: opt.warning.warnings_as_errors,
    };
    run_cmd(cargo.command()?)
}

/// Build uefi-test-runner and run it in QEMU.
fn run_vm_tests(opt: &QemuOpt) -> Result<()> {
    let mut features = vec![Feature::Qemu];
    if opt.ci {
        features.push(Feature::Ci);
    }

    // Build uefi-test-runner.
    let cargo = Cargo {
        action: CargoAction::Build,
        features,
        nightly: true,
        packages: vec![Package::UefiTestRunner],
        release: opt.build_mode.release,
        target: Some(*opt.target),
        warnings_as_errors: false,
    };
    run_cmd(cargo.command()?)?;

    qemu::run_qemu(*opt.target, opt)
}

/// Run unit tests and doctests on the host. Most of uefi-rs is tested
/// with VM tests, but a few things like macros and data types can be
/// tested with regular tests.
fn run_host_tests() -> Result<()> {
    let cargo = Cargo {
        action: CargoAction::Test,
        features: vec![Feature::Exts],
        nightly: false,
        // Don't test uefi-services (or the packages that depend on it)
        // as it has lang items that conflict with `std`. The xtask
        // currently doesn't have any tests.
        packages: vec![Package::Uefi, Package::UefiMacros, Package::Xtask],
        release: false,
        // Use the host target so that tests can run without a VM.
        target: None,
        warnings_as_errors: false,
    };
    run_cmd(cargo.command()?)
}

fn main() -> Result<()> {
    let opt = Opt::parse();

    match &opt.action {
        Action::Build(build_opt) => build(build_opt),
        Action::Clippy(clippy_opt) => clippy(clippy_opt),
        Action::Doc(doc_opt) => doc(doc_opt),
        Action::Run(qemu_opt) => run_vm_tests(qemu_opt),
        Action::Test(_) => run_host_tests(),
    }
}
