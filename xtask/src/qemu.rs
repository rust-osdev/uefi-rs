use crate::arch::UefiArch;
use crate::disk::{check_mbr_test_disk, create_mbr_test_disk};
use crate::opt::QemuOpt;
use crate::pipe::Pipe;
use crate::tpm::Swtpm;
use crate::util::command_to_string;
use crate::{net, platform};
use anyhow::{bail, Context, Result};
use regex::bytes::Regex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::env;
use std::ffi::OsString;
use std::io::{BufRead, BufReader, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use tar::Archive;
use tempfile::TempDir;
use ureq::Agent;
#[cfg(target_os = "linux")]
use {std::fs::Permissions, std::os::unix::fs::PermissionsExt};

/// Name of the ovmf-prebuilt release tag.
const OVMF_PREBUILT_TAG: &str = "edk2-stable202211-r1";

/// SHA-256 hash of the release tarball.
const OVMF_PREBUILT_HASH: &str = "b085cfe18fd674bf70a31af1dc3e991bcd25cb882981c6d3523d81260f1e0d12";

/// Directory into which the prebuilts will be download (relative to the repo root).
const OVMF_PREBUILT_DIR: &str = "target/ovmf";

/// Environment variable for overriding the path of the OVMF code file.
const ENV_VAR_OVMF_CODE: &str = "OVMF_CODE";

/// Environment variable for overriding the path of the OVMF vars file.
const ENV_VAR_OVMF_VARS: &str = "OVMF_VARS";

/// Environment variable for overriding the path of the OVMF shell file.
const ENV_VAR_OVMF_SHELL: &str = "OVMF_SHELL";

/// Download `url` and return the raw data.
fn download_url(url: &str) -> Result<Vec<u8>> {
    let agent: Agent = ureq::AgentBuilder::new()
        .user_agent("uefi-rs-ovmf-downloader")
        .build();

    // Limit the size of the download.
    let max_size_in_bytes = 4 * 1024 * 1024;

    // Download the file.
    println!("downloading {url}");
    let resp = agent.get(url).call()?;
    let mut data = Vec::with_capacity(max_size_in_bytes);
    resp.into_reader()
        .take(max_size_in_bytes.try_into().unwrap())
        .read_to_end(&mut data)?;
    println!("received {} bytes", data.len());

    Ok(data)
}

// Extract the tarball's files into `prebuilt_dir`.
//
// `tarball_data` is raw decompressed tar data.
fn extract_prebuilt(tarball_data: &[u8], prebuilt_dir: &Path) -> Result<()> {
    let cursor = Cursor::new(tarball_data);
    let mut archive = Archive::new(cursor);

    // Extract each file entry.
    for entry in archive.entries()? {
        let mut entry = entry?;

        // Skip directories.
        if entry.size() == 0 {
            continue;
        }

        let path = entry.path()?;
        // Strip the leading directory, which is the release name.
        let path: PathBuf = path.components().skip(1).collect();

        let dir = path.parent().unwrap();
        let dst_dir = prebuilt_dir.join(dir);
        let dst_path = prebuilt_dir.join(path);
        println!("unpacking to {}", dst_path.display());
        fs_err::create_dir_all(dst_dir)?;
        entry.unpack(dst_path)?;
    }

    Ok(())
}

/// Update the local copy of the prebuilt OVMF files. Does nothing if the local
/// copy is already up to date.
fn update_prebuilt() -> Result<PathBuf> {
    let prebuilt_dir = Path::new(OVMF_PREBUILT_DIR);
    let hash_path = prebuilt_dir.join("sha256");

    // Check if the hash file already has the expected hash in it. If so, assume
    // that we've already got the correct prebuilt downloaded and unpacked.
    if let Ok(current_hash) = fs_err::read_to_string(&hash_path) {
        if current_hash == OVMF_PREBUILT_HASH {
            return Ok(prebuilt_dir.to_path_buf());
        }
    }

    let base_url = "https://github.com/rust-osdev/ovmf-prebuilt/releases/download";
    let url = format!(
        "{base_url}/{release}/{release}-bin.tar.xz",
        release = OVMF_PREBUILT_TAG
    );

    let data = download_url(&url)?;

    // Validate the hash.
    let actual_hash = format!("{:x}", Sha256::digest(&data));
    if actual_hash != OVMF_PREBUILT_HASH {
        bail!(
            "file hash {actual_hash} does not match {}",
            OVMF_PREBUILT_HASH
        );
    }

    // Unpack the tarball.
    println!("decompressing tarball");
    let mut decompressed = Vec::new();
    let mut compressed = Cursor::new(data);
    lzma_rs::xz_decompress(&mut compressed, &mut decompressed)?;

    // Clear out the existing prebuilt dir, if present.
    let _ = fs_err::remove_dir_all(prebuilt_dir);

    // Extract the files.
    extract_prebuilt(&decompressed, prebuilt_dir)?;

    // Rename the x64 directory to x86_64, to match `Arch::as_str`.
    fs_err::rename(prebuilt_dir.join("x64"), prebuilt_dir.join("x86_64"))?;

    // Write out the hash file. When we upgrade to a new release of
    // ovmf-prebuilt, the hash will no longer match, triggering a fresh
    // download.
    fs_err::write(&hash_path, actual_hash)?;

    Ok(prebuilt_dir.to_path_buf())
}

#[derive(Clone, Copy, Debug)]
enum OvmfFileType {
    Code,
    Vars,
    Shell,
}

impl OvmfFileType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Vars => "vars",
            Self::Shell => "shell",
        }
    }

    fn extension(&self) -> &'static str {
        match self {
            Self::Code | Self::Vars => "fd",
            Self::Shell => "efi",
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
                var_name = ENV_VAR_OVMF_CODE;
            }
            Self::Vars => {
                opt_path = &opt.ovmf_vars;
                var_name = ENV_VAR_OVMF_VARS;
            }
            Self::Shell => {
                opt_path = &None;
                var_name = ENV_VAR_OVMF_SHELL;
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
    shell: PathBuf,
}

impl OvmfPaths {
    /// Search for an OVMF file (either code or vars).
    ///
    /// There are multiple locations where a file is searched at in the following
    /// priority:
    /// 1. Command-line arg
    /// 2. Environment variable
    /// 3. Prebuilt file (automatically downloaded)
    fn find_ovmf_file(file_type: OvmfFileType, opt: &QemuOpt, arch: UefiArch) -> Result<PathBuf> {
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
            let prebuilt_dir = update_prebuilt()?;

            Ok(prebuilt_dir.join(format!(
                "{arch}/{}.{}",
                file_type.as_str(),
                file_type.extension()
            )))
        }
    }

    /// Find path to OVMF files by the strategy documented for
    /// [`Self::find_ovmf_file`].
    fn find(opt: &QemuOpt, arch: UefiArch) -> Result<Self> {
        let code = Self::find_ovmf_file(OvmfFileType::Code, opt, arch)?;
        let vars = Self::find_ovmf_file(OvmfFileType::Vars, opt, arch)?;
        let shell = Self::find_ovmf_file(OvmfFileType::Shell, opt, arch)?;

        Ok(Self { code, vars, shell })
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
fn build_esp_dir(opt: &QemuOpt, ovmf_paths: &OvmfPaths) -> Result<PathBuf> {
    let build_mode = if opt.build_mode.release {
        "release"
    } else {
        "debug"
    };
    let build_dir = Path::new("target")
        .join(opt.target.as_triple())
        .join(build_mode);
    let esp_dir = build_dir.join("esp");

    // Create boot dir.
    let boot_dir = esp_dir.join("EFI").join("Boot");
    if !boot_dir.exists() {
        fs_err::create_dir_all(&boot_dir)?;
    }

    let boot_file_name = match *opt.target {
        UefiArch::AArch64 => "BootAA64.efi",
        UefiArch::IA32 => "BootIA32.efi",
        UefiArch::X86_64 => "BootX64.efi",
    };

    if let Some(example) = &opt.example {
        // Launch examples directly.
        let src_path = build_dir.join("examples").join(format!("{example}.efi"));
        fs_err::copy(src_path, boot_dir.join(boot_file_name))?;
    } else {
        // For the test-runner, launch the `shell_launcher` binary first. That
        // will then launch the UEFI shell, and run the `uefi-test-runner`
        // inside the shell. This allows the test-runner to test protocols that
        // use the shell.
        let shell_launcher = build_dir.join("shell_launcher.efi");
        fs_err::copy(shell_launcher, boot_dir.join(boot_file_name))?;

        fs_err::copy(&ovmf_paths.shell, boot_dir.join("shell.efi"))?;

        let test_runner = build_dir.join("uefi-test-runner.efi");
        fs_err::copy(test_runner, boot_dir.join("test_runner.efi"))?;
    };

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

    // Set the boot menu timeout to zero. On aarch64 in particular this speeds
    // up the boot a lot. Note that we have to enable the menu here even though
    // we are skipping right past it, otherwise `splash-time` is ignored in
    // favor of a hardcoded default timeout.
    cmd.args(["-boot", "menu=on,splash-time=0"]);

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
    let esp_dir = build_esp_dir(opt, &ovmf_paths)?;
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

    // Print the actual used QEMU command for running the test.
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
