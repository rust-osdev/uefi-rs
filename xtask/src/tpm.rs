// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::opt::TpmVersion;
use crate::util::command_to_string;
use anyhow::Result;
use std::process::{Child, Command};
use tempfile::TempDir;

// Adapted from https://qemu.readthedocs.io/en/latest/specs/tpm.html

/// Wrapper for running `swtpm`, a software TPM emulator.
///
/// <https://github.com/stefanberger/swtpm>
///
/// The process is killed on drop.
pub struct Swtpm {
    tmp_dir: TempDir,
    child: Child,
}

impl Swtpm {
    /// Run `swtpm` in a new process.
    pub fn spawn(version: TpmVersion) -> Result<Self> {
        let tmp_dir = TempDir::new()?;
        let tmp_path = tmp_dir.path().to_str().unwrap();

        let mut cmd = Command::new("swtpm");
        cmd.args([
            "socket",
            "--tpmstate",
            &format!("dir={tmp_path}"),
            "--ctrl",
            &format!("type=unixio,path={tmp_path}/swtpm-sock"),
            // Terminate when the connection drops. If for any reason
            // this fails, the process will be killed on drop.
            "--terminate",
            // Hide some log spam.
            "--log",
            "file=-",
        ]);

        if version == TpmVersion::V2 {
            cmd.arg("--tpm2");
        }

        println!("{}", command_to_string(&cmd));
        let child = cmd.spawn()?;

        Ok(Self { tmp_dir, child })
    }

    /// Get the QEMU args needed to connect to the TPM emulator.
    pub fn qemu_args(&self) -> Vec<String> {
        let socket_path = self.tmp_dir.path().join("swtpm-sock");
        vec![
            "-chardev".into(),
            format!("socket,id=chrtpm0,path={}", socket_path.to_str().unwrap()),
            "-tpmdev".into(),
            "emulator,id=tpm0,chardev=chrtpm0".into(),
            "-device".into(),
            "tpm-tis,tpmdev=tpm0".into(),
        ]
    }
}

impl Drop for Swtpm {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
