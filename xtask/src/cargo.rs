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
    UefiServices,
    UefiTestRunner,
    Xtask,
}

impl Package {
    fn as_str(self) -> &'static str {
        match self {
            Self::Uefi => "uefi",
            Self::UefiApp => "uefi_app",
            Self::UefiMacros => "uefi-macros",
            Self::UefiServices => "uefi-services",
            Self::UefiTestRunner => "uefi-test-runner",
            Self::Xtask => "xtask",
        }
    }

    /// All published packages.
    pub fn published() -> Vec<Package> {
        vec![Self::Uefi, Self::UefiMacros, Self::UefiServices]
    }

    /// All the packages except for xtask.
    pub fn all_except_xtask() -> Vec<Package> {
        vec![
            Self::Uefi,
            Self::UefiApp,
            Self::UefiMacros,
            Self::UefiServices,
            Self::UefiTestRunner,
        ]
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Feature {
    Alloc,
    Exts,
    Logger,

    Ci,
    Qemu,
}

impl Feature {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Alloc => "alloc",
            Self::Exts => "exts",
            Self::Logger => "logger",

            Self::Ci => "uefi-test-runner/ci",
            Self::Qemu => "uefi-test-runner/qemu",
        }
    }

    /// Set of features that enables more code in the root uefi crate.
    pub fn more_code() -> Vec<Self> {
        vec![Self::Alloc, Self::Exts, Self::Logger]
    }

    fn comma_separated_string(features: &[Feature]) -> String {
        features
            .iter()
            .map(|f| f.as_str())
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CargoAction {
    Build,
    Clippy,
    Doc { open: bool },
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

/// Cargo automatically sets some env vars that can prevent the
/// channel arg (e.g. "+nightly") from working. Unset them in the
/// child's environment.
pub fn fix_nested_cargo_env(cmd: &mut Command) {
    cmd.env_remove("RUSTC");
    cmd.env_remove("RUSTDOC");
    let orig_path = env::var_os("PATH").unwrap_or_default();
    cmd.env("PATH", sanitized_path(orig_path));
}

#[derive(Debug)]
pub struct Cargo {
    pub action: CargoAction,
    pub features: Vec<Feature>,
    pub toolchain: Option<String>,
    pub packages: Vec<Package>,
    pub release: bool,
    pub target: Option<UefiArch>,
    pub warnings_as_errors: bool,
}

impl Cargo {
    pub fn command(&self) -> Result<Command> {
        let mut cmd = Command::new("cargo");

        fix_nested_cargo_env(&mut cmd);

        if let Some(toolchain) = &self.toolchain {
            cmd.arg(&format!("+{}", toolchain));
        }

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
                    tool_args.extend(&["-D", "warnings"]);
                }
            }
            CargoAction::Doc { open } => {
                action = "doc";
                if self.warnings_as_errors {
                    cmd.env("RUSTDOCFLAGS", "-Dwarnings");
                }
                if open {
                    extra_args.push("--open");
                }
            }
            CargoAction::Miri => {
                cmd.env("MIRIFLAGS", "-Zmiri-tag-raw-pointers");
                action = "miri";
                sub_action = Some("test");
            }
            CargoAction::Test => {
                action = "test";
            }
        };
        cmd.arg(action);
        if let Some(sub_action) = sub_action {
            cmd.arg(sub_action);
        }

        if self.release {
            cmd.arg("--release");
        }

        if let Some(target) = self.target {
            cmd.args(&[
                "--target",
                target.as_triple(),
                "-Zbuild-std=core,compiler_builtins,alloc",
                "-Zbuild-std-features=compiler-builtins-mem",
            ]);
        }

        if self.packages.is_empty() {
            bail!("packages cannot be empty");
        }
        for package in &self.packages {
            cmd.args(&["--package", package.as_str()]);
        }

        if !self.features.is_empty() {
            cmd.args(&[
                "--features",
                &Feature::comma_separated_string(&self.features),
            ]);
        }

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
            Feature::comma_separated_string(&Feature::more_code()),
            "alloc,exts,logger"
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
            action: CargoAction::Doc { open: true },
            features: vec![Feature::Alloc],
            toolchain: Some("nightly".into()),
            packages: vec![Package::Uefi, Package::Xtask],
            release: false,
            target: None,
            warnings_as_errors: true,
        };
        assert_eq!(
            command_to_string(&cargo.command().unwrap()),
            "RUSTDOCFLAGS=-Dwarnings cargo +nightly doc --package uefi --package xtask --features alloc --open"
        );
    }
}
