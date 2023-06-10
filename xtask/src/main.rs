mod arch;
mod cargo;
mod check_raw;
mod device_path;
mod disk;
mod net;
mod opt;
mod pipe;
mod platform;
mod qemu;
mod tpm;
mod util;

use crate::opt::{FmtOpt, TestOpt};
use anyhow::Result;
use arch::UefiArch;
use cargo::{Cargo, CargoAction, Feature, Package, TargetTypes};
use clap::Parser;
use itertools::Itertools;
use opt::{Action, BuildOpt, ClippyOpt, DocOpt, Opt, QemuOpt, TpmVersion};
use std::process::Command;
use util::run_cmd;

fn build_feature_permutations(opt: &BuildOpt) -> Result<()> {
    for package in [Package::Uefi, Package::UefiServices] {
        let all_package_features = Feature::package_features(package);
        for features in all_package_features.iter().powerset() {
            let features = features.iter().map(|f| **f).collect();

            let cargo = Cargo {
                action: CargoAction::Build,
                features,
                packages: vec![package],
                release: opt.build_mode.release,
                target: Some(*opt.target),
                warnings_as_errors: true,
                target_types: TargetTypes::BinsExamplesLib,
            };
            run_cmd(cargo.command()?)?;
        }
    }

    Ok(())
}

fn build(opt: &BuildOpt) -> Result<()> {
    if opt.feature_permutations {
        return build_feature_permutations(opt);
    }

    let cargo = Cargo {
        action: CargoAction::Build,
        features: Feature::more_code(false, true),
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
        features: Feature::more_code(false, true),
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
        action: CargoAction::Doc {
            open: opt.open,
            document_private_items: opt.document_private_items,
        },
        features: Feature::more_code(*opt.unstable, true),
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
        features: [Feature::Alloc].into(),
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
    let mut features = vec![];

    // Enable the DebugSupport test on supported platforms. Not available on
    // AARCH64 since edk2 commit f4213fed34.
    if *opt.target != UefiArch::AArch64 {
        features.push(Feature::DebugSupport);
    }

    // Enable the PXE test unless networking is disabled or the arch doesn't
    // support it.
    if *opt.target == UefiArch::X86_64 && !opt.disable_network {
        features.push(Feature::Pxe);
    }

    // Enable TPM tests if a TPM device is present.
    match opt.tpm {
        Some(TpmVersion::V1) => features.push(Feature::TpmV1),
        Some(TpmVersion::V2) => features.push(Feature::TpmV2),
        None => {}
    }

    // Enable the multi-processor test if not targeting AARCH64, and if KVM is
    // available. KVM is available on Linux generally, but not in our CI.
    if *opt.target != UefiArch::AArch64 && platform::is_linux() && !opt.ci {
        features.push(Feature::MultiProcessor);
    }

    // Enable `unstable` if requested.
    if *opt.unstable {
        features.push(Feature::TestUnstable);
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
fn run_host_tests(test_opt: &TestOpt) -> Result<()> {
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

    let mut packages = vec![Package::UefiRaw, Package::Uefi];
    if !test_opt.skip_macro_tests {
        packages.push(Package::UefiMacros);
    }

    // Run uefi-rs and uefi-macros tests.
    let cargo = Cargo {
        action: CargoAction::Test,
        // At least one unit test, for make_boxed() currently, has different behaviour dependent on
        // the unstable feature. Because of this, we need to allow to test both variants. Runtime
        // features is set to no as it is not possible as as soon a #[global_allocator] is
        // registered, the Rust runtime executing the tests uses it as well.
        features: Feature::more_code(*test_opt.unstable, false),
        // Don't test uefi-services (or the packages that depend on it)
        // as it has lang items that conflict with `std`.
        packages,
        release: false,
        // Use the host target so that tests can run without a VM.
        target: None,
        warnings_as_errors: false,
        target_types: TargetTypes::Default,
    };
    run_cmd(cargo.command()?)
}

/// Formats the project: nix, rust, and yml.
fn run_fmt_project(fmt_opt: &FmtOpt) -> Result<()> {
    // fmt rust
    {
        eprintln!("Formatting: rust");
        let mut command = Command::new("cargo");
        command.arg("fmt");
        if fmt_opt.check {
            command.arg("--check");
        }
        command
            .arg("--all")
            .arg("--")
            .arg("--config")
            .arg("imports_granularity=Module");

        match run_cmd(command) {
            Ok(_) => {
                eprintln!("✅ rust files formatted")
            }
            Err(e) => {
                if fmt_opt.check {
                    eprintln!("❌ rust files do not pass check");
                } else {
                    eprintln!("❌ rust formatter failed: {e:#?}");
                }
            }
        }
    }

    // fmt yml
    if has_cmd("yamlfmt") {
        eprintln!("Formatting: yml");
        let mut command = Command::new("yamlfmt");
        if fmt_opt.check {
            command.arg("-lint");
        }
        // We only have yml files here.
        command.arg(".github");

        match run_cmd(command) {
            Ok(_) => {
                eprintln!("✅ yml files formatted")
            }
            Err(e) => {
                if fmt_opt.check {
                    eprintln!("❌ yml files do not pass check");
                } else {
                    eprintln!("❌ yml formatter failed: {e:#?}");
                }
            }
        }
    } else {
        eprintln!("Formatting: yml - SKIPPED");
    }

    // fmt nix
    if has_cmd("nixpkgs-fmt") {
        eprintln!("Formatting: nix");
        let mut command = Command::new("nixpkgs-fmt");
        if fmt_opt.check {
            command.arg("--check");
        }
        command.arg("nix");
        command.arg("shell.nix");

        match run_cmd(command) {
            Ok(_) => {
                eprintln!("✅ nix files formatted")
            }
            Err(e) => {
                if fmt_opt.check {
                    eprintln!("❌ nix files do not pass check");
                } else {
                    eprintln!("❌ nix formatter failed: {e:#?}");
                }
            }
        }
    } else {
        eprintln!("Formatting: nix - SKIPPED");
    }

    Ok(())
}

fn has_cmd(target_cmd: &str) -> bool {
    #[cfg(target_os = "windows")]
    let mut cmd = Command::new("where");
    #[cfg(target_family = "unix")]
    let mut cmd = Command::new("which");
    cmd.arg(target_cmd);
    run_cmd(cmd).is_ok()
}

fn main() -> Result<()> {
    let opt = Opt::parse();

    match &opt.action {
        Action::Build(build_opt) => build(build_opt),
        Action::CheckRaw(_) => check_raw::check_raw(),
        Action::Clippy(clippy_opt) => clippy(clippy_opt),
        Action::Doc(doc_opt) => doc(doc_opt),
        Action::GenCode(gen_opt) => device_path::gen_code(gen_opt),
        Action::Miri(_) => run_miri(),
        Action::Run(qemu_opt) => run_vm_tests(qemu_opt),
        Action::Test(test_opt) => run_host_tests(test_opt),
        Action::Fmt(fmt_opt) => run_fmt_project(fmt_opt),
    }
}
