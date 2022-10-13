mod arch;
mod cargo;
mod disk;
mod net;
mod opt;
mod pipe;
mod platform;
mod qemu;
mod util;

use anyhow::Result;
use cargo::{fix_nested_cargo_env, Cargo, CargoAction, Feature, Package, TargetTypes};
use clap::Parser;
use opt::{Action, BuildOpt, ClippyOpt, DocOpt, Opt, QemuOpt};
use std::process::Command;
use tempfile::TempDir;
use util::{command_to_string, run_cmd};

fn build(opt: &BuildOpt) -> Result<()> {
    let cargo = Cargo {
        action: CargoAction::Build,
        features: Feature::more_code(),
        packages: Package::all_except_xtask(),
        release: opt.build_mode.release,
        target: Some(*opt.target),
        warnings_as_errors: false,
        target_types: TargetTypes::BinsExamplesLib,
    };
    run_cmd(cargo.command()?)
}

fn clippy(opt: &ClippyOpt) -> Result<()> {
    // Run clippy on all the UEFI packages.
    let cargo = Cargo {
        action: CargoAction::Clippy,
        features: Feature::more_code(),
        packages: Package::all_except_xtask(),
        release: false,
        target: Some(*opt.target),
        warnings_as_errors: opt.warning.warnings_as_errors,
        target_types: TargetTypes::BinsExamplesLib,
    };
    run_cmd(cargo.command()?)?;

    // Run clippy on xtask.
    let cargo = Cargo {
        action: CargoAction::Clippy,
        features: Vec::new(),
        packages: vec![Package::Xtask],
        release: false,
        target: None,
        warnings_as_errors: opt.warning.warnings_as_errors,
        target_types: TargetTypes::Default,
    };
    run_cmd(cargo.command()?)
}

/// Build docs.
fn doc(opt: &DocOpt) -> Result<()> {
    let cargo = Cargo {
        action: CargoAction::Doc { open: opt.open },
        features: Feature::more_code(),
        packages: Package::published(),
        release: false,
        target: None,
        warnings_as_errors: opt.warning.warnings_as_errors,
        target_types: TargetTypes::Default,
    };
    run_cmd(cargo.command()?)
}

/// Run unit tests and doctests under Miri.
fn run_miri() -> Result<()> {
    let cargo = Cargo {
        action: CargoAction::Miri,
        features: [Feature::Exts].into(),
        packages: [Package::Uefi].into(),
        release: false,
        target: None,
        warnings_as_errors: false,
        target_types: TargetTypes::Default,
    };
    run_cmd(cargo.command()?)
}

/// Build uefi-test-runner and run it in QEMU.
fn run_vm_tests(opt: &QemuOpt) -> Result<()> {
    let mut features = vec![Feature::Qemu];

    // Always enable the ci feature when not building on Linux so that
    // the MP test is skipped. That test doesn't work with kvm disabled
    // (see https://github.com/rust-osdev/uefi-rs/issues/103).
    if opt.ci || !platform::is_linux() {
        features.push(Feature::Ci);
    }

    // Build uefi-test-runner.
    let cargo = Cargo {
        action: CargoAction::Build,
        features,
        packages: vec![Package::UefiTestRunner],
        release: opt.build_mode.release,
        target: Some(*opt.target),
        warnings_as_errors: false,
        target_types: TargetTypes::BinsExamples,
    };
    run_cmd(cargo.command()?)?;

    qemu::run_qemu(*opt.target, opt)
}

/// Run unit tests and doctests on the host. Most of uefi-rs is tested
/// with VM tests, but a few things like macros and data types can be
/// tested with regular tests.
fn run_host_tests() -> Result<()> {
    // Run xtask tests.
    let cargo = Cargo {
        action: CargoAction::Test,
        features: Vec::new(),
        packages: vec![Package::Xtask],
        release: false,
        target: None,
        warnings_as_errors: false,
        target_types: TargetTypes::Default,
    };
    run_cmd(cargo.command()?)?;

    // Run uefi-rs and uefi-macros tests.
    let cargo = Cargo {
        action: CargoAction::Test,
        features: vec![Feature::Exts],
        // Don't test uefi-services (or the packages that depend on it)
        // as it has lang items that conflict with `std`.
        packages: vec![Package::Uefi, Package::UefiMacros],
        release: false,
        // Use the host target so that tests can run without a VM.
        target: None,
        warnings_as_errors: false,
        target_types: TargetTypes::Default,
    };
    run_cmd(cargo.command()?)
}

/// Test that the template app builds successfully with the released
/// versions of the libraries on crates.io.
///
/// The `build` action also builds the template app, but due to the
/// `patch.crates-io` of the top-level Cargo.toml the app is built using
/// the current versions of the libraries in this repo. To give warning
/// when the latest crates.io releases of the libraries are broken (due
/// to changes in the nightly toolchain), this action copies the
/// template to a temporary directory and builds it in isolation.
///
/// The build command is also checked against the contents of
/// `BUILDING.md` to ensure that the doc correctly describes how to
/// build an app.
fn test_latest_release() -> Result<()> {
    // Recursively copy the template app to a temporary directory. This
    // isolates the app from the full git repo so that the
    // `patch.crates-io` section of the root Cargo.toml doesn't apply.
    let tmp_dir = TempDir::new()?;
    let tmp_dir = tmp_dir.path();
    let mut cp_cmd = Command::new("cp");
    cp_cmd
        .args(["--recursive", "--verbose", "template"])
        .arg(tmp_dir);
    run_cmd(cp_cmd)?;

    // Create cargo build command, not using the `cargo` module to make
    // it explicit that it matches the command in `BUILDING.md`.
    let mut build_cmd = Command::new("cargo");
    fix_nested_cargo_env(&mut build_cmd);
    build_cmd
        .args(["build", "--target", "x86_64-unknown-uefi"])
        .current_dir(tmp_dir.join("template"));

    // Check that the command is indeed in BUILDING.md, then verify the
    // build succeeds.
    let building_md = include_str!("../../BUILDING.md");
    assert!(building_md.contains(&command_to_string(&build_cmd)));
    run_cmd(build_cmd)
}

fn main() -> Result<()> {
    let opt = Opt::parse();

    match &opt.action {
        Action::Build(build_opt) => build(build_opt),
        Action::Clippy(clippy_opt) => clippy(clippy_opt),
        Action::Doc(doc_opt) => doc(doc_opt),
        Action::Miri(_) => run_miri(),
        Action::Run(qemu_opt) => run_vm_tests(qemu_opt),
        Action::Test(_) => run_host_tests(),
        Action::TestLatestRelease(_) => test_latest_release(),
    }
}
