use crate::arch::UefiArch;
use crate::disk::{check_mbr_test_disk, create_mbr_test_disk};
use crate::opt::QemuOpt;
use crate::pipe::Pipe;
use crate::tpm::Swtpm;
use crate::util::command_to_string;
use crate::{net, platform};
use anyhow::{anyhow, bail, Context, Result};
use regex::bytes::Regex;
use serde_json::{json, Value};
use std::env;
use std::ffi::OsString;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use tempfile::TempDir;
#[cfg(target_os = "linux")]
use {std::fs::Permissions, std::os::unix::fs::PermissionsExt};

#[derive(Clone, Copy, Debug)]
enum OvmfFileType {
    Code,
    Vars,
}

impl OvmfFileType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Vars => "vars",
        }
    }

    /// Get a user-provided path for the given OVMF file type.
    ///
    /// This uses the command-line arg if present, otherwise it falls back to an
    /// environment variable. If neither is present, returns `None`.
    fn get_user_provided_path(self, opt: &QemuOpt) -> Option<PathBuf> {
        let opt_path;
        let var_name;
        match self {
            Self::Code => {
                opt_path = &opt.ovmf_code;
                var_name = "OVMF_CODE";
            }
            Self::Vars => {
                opt_path = &opt.ovmf_vars;
                var_name = "OVMF_VARS";
            }
        }
        if let Some(path) = opt_path {
            Some(path.clone())
        } else {
            env::var_os(var_name).map(PathBuf::from)
        }
    }
}

struct OvmfPaths {
    code: PathBuf,
    vars: PathBuf,
}

impl OvmfPaths {
    /// If OVMF files can not or should not be found at well-known locations,
    /// this optional environment variable can point to it.
    ///
    /// This variable points to the `_CODE.fd` file.

    const ENV_VAR_OVMF_CODE: &'static str = "OVMF_CODE";
    /// If OVMF files can not or should not be found at well-known locations,
    /// this optional environment variable can point to it.
    ///
    /// This variable points to the `_VARS.fd` file.
    const ENV_VAR_OVMF_VARS: &'static str = "OVMF_VARS";

    fn get_path(&self, file_type: OvmfFileType) -> &Path {
        match file_type {
            OvmfFileType::Code => &self.code,
            OvmfFileType::Vars => &self.vars,
        }
    }

    /// Get the Arch Linux OVMF paths for the given guest arch.
    fn arch_linux(arch: UefiArch) -> Self {
        match arch {
            // Package "edk2-armvirt".
            UefiArch::AArch64 => Self {
                code: "/usr/share/edk2-armvirt/aarch64/QEMU_CODE.fd".into(),
                vars: "/usr/share/edk2-armvirt/aarch64/QEMU_VARS.fd".into(),
            },
            // Package "edk2-ovmf".
            UefiArch::IA32 => Self {
                code: "/usr/share/edk2-ovmf/ia32/OVMF_CODE.fd".into(),
                vars: "/usr/share/edk2-ovmf/ia32/OVMF_VARS.fd".into(),
            },
            // Package "edk2-ovmf".
            UefiArch::X86_64 => Self {
                code: "/usr/share/edk2-ovmf/x64/OVMF_CODE.fd".into(),
                vars: "/usr/share/edk2-ovmf/x64/OVMF_VARS.fd".into(),
            },
        }
    }

    /// Get the CentOS OVMF paths for the given guest arch.
    fn centos_linux(arch: UefiArch) -> Option<Self> {
        match arch {
            // Package "edk2-aarch64".
            UefiArch::AArch64 => Some(Self {
                code: "/usr/share/edk2/aarch64/QEMU_EFI-pflash.raw".into(),
                vars: "/usr/share/edk2/aarch64/vars-template-pflash.raw".into(),
            }),
            // There's no official ia32 package.
            UefiArch::IA32 => None,
            // Package "edk2-ovmf".
            UefiArch::X86_64 => Some(Self {
                // Use the `.secboot` variant because the CentOS package
                // doesn't have a plain "OVMF_CODE.fd".
                code: "/usr/share/edk2/ovmf/OVMF_CODE.secboot.fd".into(),
                vars: "/usr/share/edk2/ovmf/OVMF_VARS.fd".into(),
            }),
        }
    }

    /// Get the Debian OVMF paths for the given guest arch. These paths
    /// also work on Ubuntu.
    fn debian_linux(arch: UefiArch) -> Self {
        match arch {
            // Package "qemu-efi-aarch64".
            UefiArch::AArch64 => Self {
                code: "/usr/share/AAVMF/AAVMF_CODE.fd".into(),
                vars: "/usr/share/AAVMF/AAVMF_VARS.fd".into(),
            },
            // Package "ovmf-ia32".
            UefiArch::IA32 => Self {
                code: "/usr/share/OVMF/OVMF32_CODE_4M.secboot.fd".into(),
                vars: "/usr/share/OVMF/OVMF32_VARS_4M.fd".into(),
            },
            // Package "ovmf".
            UefiArch::X86_64 => Self {
                code: "/usr/share/OVMF/OVMF_CODE.fd".into(),
                vars: "/usr/share/OVMF/OVMF_VARS.fd".into(),
            },
        }
    }

    /// Get the Fedora OVMF paths for the given guest arch.
    fn fedora_linux(arch: UefiArch) -> Self {
        match arch {
            // Package "edk2-aarch64".
            UefiArch::AArch64 => Self {
                code: "/usr/share/edk2/aarch64/QEMU_EFI-pflash.raw".into(),
                vars: "/usr/share/edk2/aarch64/vars-template-pflash.raw".into(),
            },
            // Package "edk2-ovmf-ia32".
            UefiArch::IA32 => Self {
                code: "/usr/share/edk2/ovmf-ia32/OVMF_CODE.fd".into(),
                vars: "/usr/share/edk2/ovmf-ia32/OVMF_VARS.fd".into(),
            },
            // Package "edk2-ovmf".
            UefiArch::X86_64 => Self {
                code: "/usr/share/edk2/ovmf/OVMF_CODE.fd".into(),
                vars: "/usr/share/edk2/ovmf/OVMF_VARS.fd".into(),
            },
        }
    }

    /// If a user uses NixOS, this function returns an error if the user didn't
    /// set the environment variables `OVMF_CODE` and `OVMF_VARS`.
    ///
    /// It returns nothing as the environment variables are resolved at a
    /// higher level. NixOS doesn't have globally installed software (without
    /// hacky and non-idiomatic workarounds).
    fn assist_nixos_users() -> Result<()> {
        let os_info = os_info::get();
        if os_info.os_type() == os_info::Type::NixOS {
            let code = env::var_os(Self::ENV_VAR_OVMF_CODE);
            let vars = env::var_os(Self::ENV_VAR_OVMF_VARS);
            if !matches!((code, vars), (Some(_), Some(_))) {
                return Err(anyhow!("Run `$ nix-shell` for OVMF files."));
            }
        }
        Ok(())
    }

    /// Get the Windows OVMF paths for the given guest arch.
    fn windows(arch: UefiArch) -> Self {
        match arch {
            UefiArch::AArch64 => Self {
                code: r"C:\Program Files\qemu\share\edk2-aarch64-code.fd".into(),
                vars: r"C:\Program Files\qemu\share\edk2-arm-vars.fd".into(),
            },
            UefiArch::IA32 => Self {
                code: r"C:\Program Files\qemu\share\edk2-i386-code.fd".into(),
                vars: r"C:\Program Files\qemu\share\edk2-i386-vars.fd".into(),
            },
            UefiArch::X86_64 => Self {
                code: r"C:\Program Files\qemu\share\edk2-x86_64-code.fd".into(),
                // There's no x86_64 vars file, but the i386 one works.
                vars: r"C:\Program Files\qemu\share\edk2-i386-vars.fd".into(),
            },
        }
    }

    /// Get candidate paths where OVMF code/vars might exist for the
    /// given guest arch and host platform.
    fn get_candidate_paths(arch: UefiArch) -> Result<Vec<Self>> {
        let mut candidates = Vec::new();
        if platform::is_linux() {
            candidates.push(Self::arch_linux(arch));
            if let Some(candidate) = Self::centos_linux(arch) {
                candidates.push(candidate);
            }
            candidates.push(Self::debian_linux(arch));
            candidates.push(Self::fedora_linux(arch));
            Self::assist_nixos_users()?;
        }
        if platform::is_windows() {
            candidates.push(Self::windows(arch));
        }
        Ok(candidates)
    }

    /// Search for an OVMF file (either code or vars).
    ///
    /// There are multiple locations where a file is searched at in the following
    /// priority:
    /// 1. User-defined location: See [`OvmfFileType::get_user_provided_path`]
    /// 2. Well-known location of common Linux distributions by using the
    ///    paths in `candidates`.
    fn find_ovmf_file(
        file_type: OvmfFileType,
        opt: &QemuOpt,
        candidates: &[Self],
    ) -> Result<PathBuf> {
        if let Some(path) = file_type.get_user_provided_path(opt) {
            // The user provided an exact path to use; verify that it
            // exists.
            if path.exists() {
                Ok(path)
            } else {
                bail!(
                    "ovmf {} file does not exist: {}",
                    file_type.as_str(),
                    path.display()
                );
            }
        } else {
            for candidate in candidates {
                let path = candidate.get_path(file_type);
                if path.exists() {
                    return Ok(path.to_owned());
                }
            }

            bail!(
                "no ovmf {} file found in candidates: {:?}",
                file_type.as_str(),
                candidates
                    .iter()
                    .map(|c| c.get_path(file_type))
                    .collect::<Vec<_>>(),
            );
        }
    }

    /// Find path to OVMF files by the strategy documented for
    /// [`Self::find_ovmf_file`].
    fn find(opt: &QemuOpt, arch: UefiArch) -> Result<Self> {
        let candidates = Self::get_candidate_paths(arch)?;

        let code = Self::find_ovmf_file(OvmfFileType::Code, opt, &candidates)?;
        let vars = Self::find_ovmf_file(OvmfFileType::Vars, opt, &candidates)?;

        Ok(Self { code, vars })
    }
}

enum PflashMode {
    ReadOnly,
    ReadWrite,
}

fn add_pflash_args(cmd: &mut Command, file: &Path, mode: PflashMode) {
    // Build the argument as an OsString to avoid requiring a UTF-8 path.
    let mut arg = OsString::from("if=pflash,format=raw,readonly=");
    arg.push(match mode {
        PflashMode::ReadOnly => "on",
        PflashMode::ReadWrite => "off",
    });
    arg.push(",file=");
    arg.push(file);

    cmd.arg("-drive");
    cmd.arg(arg);
}

pub struct Io {
    reader: BufReader<Box<dyn Read + Send>>,
    writer: Box<dyn Write + Send>,
}

impl Io {
    pub fn new<R, W>(r: R, w: W) -> Self
    where
        R: Read + Send + 'static,
        W: Write + Send + 'static,
    {
        Self {
            reader: BufReader::new(Box::new(r)),
            writer: Box::new(w),
        }
    }

    fn read_line(&mut self) -> Result<String> {
        let mut line = String::new();
        let num = self.reader.read_line(&mut line)?;
        if num == 0 {
            bail!("EOF reached");
        }
        Ok(line)
    }

    fn read_json(&mut self) -> Result<Value> {
        let line = self.read_line()?;
        Ok(serde_json::from_str(&line)?)
    }

    fn write_all(&mut self, s: &str) -> Result<()> {
        self.writer.write_all(s.as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }

    fn write_json(&mut self, json: Value) -> Result<()> {
        // Note: it's important not to add anything after the JSON data
        // such as a trailing newline. On Windows, QEMU's pipe reader
        // will hang if that happens.
        self.write_all(&json.to_string())
    }
}

fn process_qemu_io(mut monitor_io: Io, mut serial_io: Io, tmp_dir: &Path) -> Result<()> {
    let mut tests_complete = false;

    // This regex is used to detect and strip ANSI escape codes. These
    // escapes are added by the console output protocol when writing to
    // the serial device.
    let ansi_escape = Regex::new(r"(\x9b|\x1b\[)[0-?]*[ -/]*[@-~]").expect("invalid regex");

    // Execute the QEMU monitor handshake, doing basic sanity checks.
    assert!(monitor_io.read_line()?.starts_with(r#"{"QMP":"#));
    monitor_io.write_json(json!({"execute": "qmp_capabilities"}))?;
    assert_eq!(monitor_io.read_json()?, json!({"return": {}}));

    while let Ok(line) = serial_io.read_line() {
        // Strip whitespace and ANSI escape codes.
        let line = line.trim_end();
        let line = ansi_escape.replace_all(line.as_bytes(), &b""[..]);
        let line = String::from_utf8(line.into()).expect("line is not utf8");

        // Send an "OK" response to the app.
        let mut reply_ok = || serial_io.write_all("OK\n");

        // If the app requests a screenshot, take it.
        if let Some(reference_name) = line.strip_prefix("SCREENSHOT: ") {
            let screenshot_path = tmp_dir.join("screenshot.ppm");

            // Ask QEMU to take a screenshot.
            monitor_io.write_json(json!({
                "execute": "screendump",
                "arguments": {"filename": screenshot_path}}
            ))?;

            // Wait for QEMU's acknowledgement, ignoring events.
            let mut reply = monitor_io.read_json()?;
            while reply.as_object().unwrap().contains_key("event") {
                reply = monitor_io.read_json()?;
            }
            assert_eq!(reply, json!({"return": {}}));

            // Tell the VM that the screenshot was taken
            reply_ok()?;

            // Compare screenshot to the reference file specified by the user.
            // TODO: Add an operating mode where the reference is created if it doesn't exist.
            let reference_file =
                Path::new("uefi-test-runner/screenshots").join(format!("{reference_name}.ppm"));
            let expected = fs_err::read(reference_file)?;
            let actual = fs_err::read(&screenshot_path)?;
            // Use `assert` rather than `assert_eq` here to avoid
            // dumping a huge amount of raw pixel data on failure.
            assert!(
                expected == actual,
                "screenshot does not match reference image"
            )
        } else if line == "TESTS_COMPLETE" {
            // The app sends this command after running its tests to
            // indicate it actually got to the end. If the tests failed
            // earlier with a panic, this command will never be
            // received.
            tests_complete = true;

            reply_ok()?;
        } else {
            println!("{line}");
        }
    }

    if !tests_complete {
        bail!("tests did not complete successfully");
    }

    Ok(())
}

/// Create an EFI boot directory to pass into QEMU.
fn build_esp_dir(opt: &QemuOpt) -> Result<PathBuf> {
    let build_mode = if opt.build_mode.release {
        "release"
    } else {
        "debug"
    };
    let build_dir = Path::new("target")
        .join(opt.target.as_triple())
        .join(build_mode);
    let esp_dir = build_dir.join("esp");
    let boot_dir = esp_dir.join("EFI").join("Boot");
    let built_file = if let Some(example) = &opt.example {
        build_dir.join("examples").join(format!("{example}.efi"))
    } else {
        build_dir.join("uefi-test-runner.efi")
    };
    let output_file = match *opt.target {
        UefiArch::AArch64 => "BootAA64.efi",
        UefiArch::IA32 => "BootIA32.efi",
        UefiArch::X86_64 => "BootX64.efi",
    };
    if !boot_dir.exists() {
        fs_err::create_dir_all(&boot_dir)?;
    }
    fs_err::copy(built_file, boot_dir.join(output_file))?;

    // Add a test file that is used in the media protocol tests.
    fs_err::write(boot_dir.join("test_input.txt"), "test input data")?;

    Ok(esp_dir)
}

/// Wrap a child process to automatically kill it when dropped.
struct ChildWrapper(Child);

impl Drop for ChildWrapper {
    fn drop(&mut self) {
        // Do nothing if child has already exited (this call doesn't block).
        if matches!(self.0.try_wait(), Ok(Some(_))) {
            return;
        }

        // Try to stop the process, then wait for it to exit. Log errors
        // but otherwise ignore.
        if let Err(err) = self.0.kill() {
            eprintln!("failed to kill process: {err}");
        }
        if let Err(err) = self.0.wait() {
            eprintln!("failed to wait for process exit: {err}");
        }
    }
}

pub fn run_qemu(arch: UefiArch, opt: &QemuOpt) -> Result<()> {
    let esp_dir = build_esp_dir(opt)?;

    let qemu_exe = match arch {
        UefiArch::AArch64 => "qemu-system-aarch64",
        UefiArch::IA32 | UefiArch::X86_64 => "qemu-system-x86_64",
    };
    let mut cmd = Command::new(qemu_exe);

    if platform::is_windows() {
        // The QEMU installer for Windows does not automatically add the
        // directory containing the QEMU executables to the PATH. Add
        // the default directory to the PATH to make it more likely that
        // QEMU will be found on Windows. (The directory is appended, so
        // if a different directory on the PATH already has the QEMU
        // binary this change won't affect anything.)
        let mut path = env::var_os("PATH").unwrap_or_default();
        path.push(r";C:\Program Files\qemu");
        cmd.env("PATH", path);
    }

    // Disable default devices.
    // QEMU by defaults enables a ton of devices which slow down boot.
    cmd.arg("-nodefaults");

    cmd.args(["-device", "virtio-rng-pci"]);

    match arch {
        UefiArch::AArch64 => {
            // Use a generic ARM environment. Sadly qemu can't emulate a
            // RPi 4 like machine though.
            cmd.args(["-machine", "virt"]);

            // A72 is a very generic 64-bit ARM CPU in the wild.
            cmd.args(["-cpu", "cortex-a72"]);

            // Graphics device.
            cmd.args(["-device", "virtio-gpu-pci"]);
        }
        UefiArch::IA32 | UefiArch::X86_64 => {
            // Use a modern machine.
            cmd.args(["-machine", "q35"]);

            // Multi-processor services protocol test needs exactly 4 CPUs.
            cmd.args(["-smp", "4"]);

            // Allocate some memory.
            cmd.args(["-m", "256M"]);

            // Graphics device.
            cmd.args(["-vga", "std"]);

            // Enable hardware-accelerated virtualization if possible.
            if platform::is_linux() && !opt.disable_kvm && !opt.ci {
                cmd.arg("--enable-kvm");
            }

            // Exit instead of rebooting in the CI.
            if opt.ci {
                cmd.arg("-no-reboot");
            }

            // Map the QEMU exit signal to port f4.
            cmd.args(["-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]);

            // OVMF debug builds can output information to a serial `debugcon`.
            // Only enable when debugging UEFI boot.
            // cmd.args([
            //     "-debugcon",
            //     "file:debug.log",
            //     "-global",
            //     "isa-debugcon.iobase=0x402",
            // ]);
        }
    }

    let tmp_dir = TempDir::new()?;
    let tmp_dir = tmp_dir.path();

    // Set up OVMF.
    let ovmf_paths = OvmfPaths::find(opt, arch)?;

    // Make a copy of the OVMF vars file so that it can be used
    // read+write without modifying the original. Under AArch64, some
    // versions of OVMF won't boot if the vars file isn't writeable.
    let ovmf_vars = tmp_dir.join("ovmf_vars");
    fs_err::copy(&ovmf_paths.vars, &ovmf_vars)?;
    // Necessary, as for example on NixOS, the files are read-only inside
    // the Nix store.
    #[cfg(target_os = "linux")]
    fs_err::set_permissions(&ovmf_vars, Permissions::from_mode(0o666))?;

    add_pflash_args(&mut cmd, &ovmf_paths.code, PflashMode::ReadOnly);
    add_pflash_args(&mut cmd, &ovmf_vars, PflashMode::ReadWrite);

    // Mount a local directory as a FAT partition.
    cmd.arg("-drive");
    let mut drive_arg = OsString::from("format=raw,file=fat:rw:");
    drive_arg.push(esp_dir);
    cmd.arg(drive_arg);

    if opt.headless {
        cmd.args(["-display", "none"]);
    }

    let test_disk = tmp_dir.join("test_disk.fat.img");
    create_mbr_test_disk(&test_disk)?;

    cmd.arg("-drive");
    let mut drive_arg = OsString::from("format=raw,file=");
    drive_arg.push(test_disk.clone());
    cmd.arg(drive_arg);

    let qemu_monitor_pipe = Pipe::new(tmp_dir, "qemu-monitor")?;
    let serial_pipe = Pipe::new(tmp_dir, "serial")?;

    // Open a serial device connected to stdio. This is used for
    // printing logs and to receive and reply to commands.
    cmd.args(["-serial", serial_pipe.qemu_arg()]);

    // Map the QEMU monitor to a pair of named pipes
    cmd.args(["-qmp", qemu_monitor_pipe.qemu_arg()]);

    // Attach network device with DHCP configured for PXE. Skip this for
    // examples since it slows down the boot some.
    let echo_service = if !opt.disable_network && opt.example.is_none() {
        cmd.args([
            "-nic",
            "user,model=e1000,net=192.168.17.0/24,tftp=uefi-test-runner/tftp/,bootfile=fake-boot-file",
        ]);
        Some(net::EchoService::start())
    } else {
        None
    };

    // Set up a software TPM if requested.
    let _tpm = if let Some(tpm_version) = opt.tpm {
        let tpm = Swtpm::spawn(tpm_version)?;
        cmd.args(tpm.qemu_args());
        Some(tpm)
    } else {
        None
    };

    println!("{}", command_to_string(&cmd));

    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    let mut child = ChildWrapper(cmd.spawn().context("failed to launch qemu")?);

    let monitor_io = qemu_monitor_pipe.open_io()?;
    let serial_io = serial_pipe.open_io()?;

    // Capture the result to check it, but first wait for the child to
    // exit.
    let res = process_qemu_io(monitor_io, serial_io, tmp_dir);
    let status = child.0.wait()?;

    if let Some(echo_service) = echo_service {
        echo_service.stop();
    }

    // Propagate earlier error if necessary.
    res?;

    // Get qemu's exit code if possible, or return an error if
    // terminated by a signal.
    let qemu_exit_code = status
        .code()
        .context(format!("qemu was terminated by a signal: {status:?}"))?;

    let successful_exit_code = match arch {
        UefiArch::AArch64 | UefiArch::IA32 => 0,

        // The x86_64 version of uefi-test-runner uses exit code 3 to
        // indicate success. See the `shutdown` function in
        // uefi-test-runner for more details.
        UefiArch::X86_64 => 3,
    };

    if qemu_exit_code != successful_exit_code {
        bail!(
            "qemu exited with code {}, expected {}",
            qemu_exit_code,
            successful_exit_code
        );
    }

    check_mbr_test_disk(&test_disk)?;

    Ok(())
}
