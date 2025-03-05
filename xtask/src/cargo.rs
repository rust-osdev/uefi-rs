// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::arch::UefiArch;
use anyhow::{bail, Result};
use std::env;
use std::ffi::OsString;
use std::process::Command;

#[derive(Clone, Copy, Debug)]
pub enum Package {
    Uefi,
    UefiApp,
    UefiMacros,
    UefiRaw,
    UefiTestRunner,
    Xtask,
}

impl Package {
    /// Get package name.
    fn name(self) -> &'static str {
        match self {
            Self::Uefi => "uefi",
            Self::UefiApp => "uefi_app",
            Self::UefiMacros => "uefi-macros",
            Self::UefiRaw => "uefi-raw",
            Self::UefiTestRunner => "uefi-test-runner",
            Self::Xtask => "xtask",
        }
    }

    /// All published packages, in the order that publishing should occur.
    pub fn published() -> Vec<Package> {
        vec![Self::UefiRaw, Self::UefiMacros, Self::Uefi]
    }

    /// All the packages except for xtask.
    pub fn all_except_xtask() -> Vec<Package> {
        vec![
            Self::Uefi,
            Self::UefiApp,
            Self::UefiMacros,
            Self::UefiRaw,
            Self::UefiTestRunner,
        ]
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Feature {
    // `uefi` features.
    Alloc,
    GlobalAllocator,
    LogDebugcon,
    Logger,
    Unstable,
    PanicHandler,
    Qemu,

    // `uefi-test-runner` features.
    DebugSupport,
    MultiProcessor,
    Pxe,
    TestUnstable,
    TpmV1,
    TpmV2,
}

impl Feature {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Alloc => "alloc",
            Self::GlobalAllocator => "global_allocator",
            Self::LogDebugcon => "log-debugcon",
            Self::Logger => "logger",
            Self::Unstable => "unstable",
            Self::PanicHandler => "panic_handler",
            Self::Qemu => "qemu",

            Self::DebugSupport => "uefi-test-runner/debug_support",
            Self::MultiProcessor => "uefi-test-runner/multi_processor",
            Self::Pxe => "uefi-test-runner/pxe",
            Self::TestUnstable => "uefi-test-runner/unstable",
            Self::TpmV1 => "uefi-test-runner/tpm_v1",
            Self::TpmV2 => "uefi-test-runner/tpm_v2",
        }
    }

    /// Get the features for the given package.
    pub fn package_features(package: Package) -> Vec<Self> {
        match package {
            Package::Uefi => vec![
                Self::Alloc,
                Self::GlobalAllocator,
                Self::LogDebugcon,
                Self::Logger,
                Self::Unstable,
                Self::PanicHandler,
                Self::Qemu,
            ],
            Package::UefiTestRunner => {
                vec![
                    Self::DebugSupport,
                    Self::MultiProcessor,
                    Self::Pxe,
                    Self::TestUnstable,
                    Self::TpmV1,
                    Self::TpmV2,
                ]
            }
            _ => vec![],
        }
    }

    /// Set of features that enables more code in the root uefi crate.
    /// # Parameters
    /// - `include_unstable` - add all functionality behind the `unstable` feature
    /// - `runtime_features` - add all functionality that effect the runtime of Rust
    pub fn more_code(include_unstable: bool, runtime_features: bool) -> Vec<Self> {
        let mut base_features = vec![Self::Alloc, Self::LogDebugcon, Self::Logger];
        if include_unstable {
            base_features.extend([Self::Unstable])
        }
        if runtime_features {
            base_features.extend([Self::GlobalAllocator])
        }
        base_features
    }

    fn comma_separated_string(features: &[Feature]) -> String {
        features
            .iter()
            .map(|f| f.as_str())
            .collect::<Vec<_>>()
            .join(",")
    }
}

/// Select which target types (e.g. libs, bins, and examples) to include.
///
/// Cargo commands such as `build` and `clippy` include libs, bins, and
/// tests by default, but not examples. To include examples, which we
/// need for uefi-test-runner, you have to add `--examples`. But adding
/// that flag also turns off the other types, so if you specify one type
/// you have to also specify all the other types you care about.
///
/// Making things slightly tricker is that cargo will fail if a target
/// type is specified that is not present in any of the selected
/// packages. Since we run some cargo commands on a subset of packages,
/// we can't always use the same set of target types. There is an
/// `--all-targets` flag which is smarter about this, but it will enable
/// the test target which fails to compile on the UEFI targets, so we
/// can't use that either.
///
/// So to get everything working and include coverage of the examples,
/// allow each cargo invocation to specify if it wants the default set
/// of types or some more specific combo.
#[derive(Clone, Copy, Debug)]
pub enum TargetTypes {
    /// Use this to not specify any target types in the cargo command
    /// line; this will enable bins, libs, and tests if they are present.
    Default,

    /// Build bins and examples.
    BinsExamples,

    /// Build bins, examples, and libs.
    BinsExamplesLib,
}

impl TargetTypes {
    const fn args(self) -> &'static [&'static str] {
        match self {
            TargetTypes::Default => &[],
            TargetTypes::BinsExamples => &["--bins", "--examples"],
            TargetTypes::BinsExamplesLib => &[
                "--bins",
                "--examples",
                // This flag is not plural like the others because a
                // package can only include one lib.
                "--lib",
            ],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CargoAction {
    Build,
    Clippy,
    Coverage {
        lcov: bool,
        open: bool,
    },
    Doc {
        open: bool,
        document_private_items: bool,
    },
    Miri,
    Test,
}

/// Get a modified PATH to remove entries added by rustup. This is
/// necessary on Windows, see
/// https://github.com/rust-lang/rustup/issues/3031.
fn sanitized_path(orig_path: OsString) -> OsString {
    // Modify the PATH to remove entries added by rustup. This is
    // necessary on Windows, see https://github.com/rust-lang/rustup/issues/3031.
    let paths = env::split_paths(&orig_path);
    let sanitized_paths = paths.filter(|path| {
        !path
            .components()
            .any(|component| component.as_os_str() == ".rustup")
    });

    env::join_paths(sanitized_paths).expect("invalid PATH")
}

/// Update a command env to make nested cargo invocations work correctly.
///
/// Cargo automatically sets some env vars that cause problems with nested cargo
/// invocations. In particular, these env vars can:
/// * Cause unwanted rebuilds, e.g. running `cargo xtask test` multiple times
///   will rebuild every time because cached dependencies are marked as stale.
/// * Prevent channels args (e.g. "+nightly") from working.
///
/// Some related issues:
/// * <https://github.com/rust-lang/cargo/issues/15099>
/// * <https://github.com/rust-lang/rustup/issues/3031>
fn fix_nested_cargo_env(cmd: &mut Command) {
    cmd.env_remove("RUSTC");
    cmd.env_remove("RUSTDOC");

    // Clear all vars starting with `CARGO`.
    for (name, _) in env::vars() {
        if name.starts_with("CARGO") {
            cmd.env_remove(name);
        }
    }

    let orig_path = env::var_os("PATH").unwrap_or_default();
    cmd.env("PATH", sanitized_path(orig_path));
}

#[derive(Debug)]
pub struct Cargo {
    pub action: CargoAction,
    pub features: Vec<Feature>,
    pub packages: Vec<Package>,
    pub release: bool,
    pub target: Option<UefiArch>,
    pub warnings_as_errors: bool,
    pub target_types: TargetTypes,
}

impl Cargo {
    pub fn command(&self) -> Result<Command> {
        let mut cmd = Command::new("cargo");

        fix_nested_cargo_env(&mut cmd);

        let action;
        let mut sub_action = None;
        let mut extra_args: Vec<&str> = Vec::new();
        let mut tool_args: Vec<&str> = Vec::new();
        match self.action {
            CargoAction::Build => {
                action = "build";
            }
            CargoAction::Clippy => {
                action = "clippy";
                if self.warnings_as_errors {
                    tool_args.extend(["-D", "warnings"]);
                }
            }
            CargoAction::Coverage { lcov, open } => {
                action = "llvm-cov";
                if lcov {
                    extra_args.extend(["--lcov", "--output-path", "target/lcov"]);
                } else {
                    extra_args.push("--html");
                }
                if open {
                    extra_args.push("--open");
                }
            }
            CargoAction::Doc {
                open,
                document_private_items,
            } => {
                action = "doc";
                extra_args.push("--no-deps");
                if self.warnings_as_errors {
                    cmd.env("RUSTDOCFLAGS", "-Dwarnings");
                }
                if document_private_items {
                    extra_args.push("--document-private-items");
                }
                if open {
                    extra_args.push("--open");
                }
            }
            CargoAction::Miri => {
                action = "miri";
                sub_action = Some("test");
                cmd.env("MIRIFLAGS", "-Zmiri-strict-provenance");
            }
            CargoAction::Test => {
                action = "test";

                // Ensure that uefi-macros trybuild tests work regardless of
                // whether RUSTFLAGS is set.
                //
                // The trybuild tests run `rustc` with `--verbose`, which
                // affects error output. This flag is set via
                // `--config=build.rustflags` [1], but that will be ignored if
                // the RUSTFLAGS env var is set. Compensate by appending
                // `--verbose` to the var in that case.
                //
                // [1]: https://github.com/dtolnay/trybuild/blob/b1b7064b7ad11e0ab563e9eb843651d86e4545b7/src/cargo.rs#L44
                if let Ok(mut rustflags) = env::var("RUSTFLAGS") {
                    rustflags.push_str(" --verbose");
                    cmd.env("RUSTFLAGS", rustflags);
                }
            }
        };
        cmd.arg(action);
        if let Some(sub_action) = sub_action {
            cmd.arg(sub_action);
        }

        // Turn off default features so that `--feature-permutations` can test
        // with each feature enabled and disabled.
        cmd.arg("--no-default-features");

        if self.release {
            cmd.arg("--release");
        }

        if let Some(target) = self.target {
            cmd.args(["--target", target.as_triple()]);
        }

        if self.packages.is_empty() {
            bail!("packages cannot be empty");
        }
        for package in &self.packages {
            cmd.args(["--package", package.name()]);
        }

        if !self.features.is_empty() {
            cmd.args([
                "--features",
                &Feature::comma_separated_string(&self.features),
            ]);
        }

        cmd.args(self.target_types.args());

        cmd.args(extra_args);

        if !tool_args.is_empty() {
            cmd.arg("--");
            cmd.args(tool_args);
        }

        Ok(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::command_to_string;

    #[test]
    fn test_comma_separated_features() {
        assert_eq!(
            Feature::comma_separated_string(&Feature::more_code(false, false)),
            "alloc,log-debugcon,logger"
        );
        assert_eq!(
            Feature::comma_separated_string(&Feature::more_code(false, true)),
            "alloc,log-debugcon,logger,global_allocator"
        );
        assert_eq!(
            Feature::comma_separated_string(&Feature::more_code(true, false)),
            "alloc,log-debugcon,logger,unstable"
        );
        assert_eq!(
            Feature::comma_separated_string(&Feature::more_code(true, true)),
            "alloc,log-debugcon,logger,unstable,global_allocator"
        );
    }

    #[test]
    fn test_sanitize_path() {
        let (input, expected) = match env::consts::FAMILY {
            "unix" => ("Abc:/path/.rustup/cargo:Xyz", "Abc:Xyz"),
            "windows" => ("Abc;/path/.rustup/cargo;Xyz", "Abc;Xyz"),
            _ => unimplemented!(),
        };

        assert_eq!(sanitized_path(input.into()), expected);
    }

    #[test]
    fn test_cargo_command() {
        let cargo = Cargo {
            action: CargoAction::Doc {
                open: true,
                document_private_items: true,
            },
            features: vec![Feature::GlobalAllocator],
            packages: vec![Package::Uefi, Package::Xtask],
            release: false,
            target: None,
            warnings_as_errors: true,
            target_types: TargetTypes::Default,
        };
        assert_eq!(
            command_to_string(&cargo.command().unwrap()),
            "RUSTDOCFLAGS=-Dwarnings cargo doc --no-default-features --package uefi --package xtask --features global_allocator --no-deps --document-private-items --open"
        );
    }
}
