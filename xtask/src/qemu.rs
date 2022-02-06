use crate::arch::UefiArch;
use crate::opt::QemuOpt;
use crate::util::command_to_string;
use anyhow::{bail, Result};
use fs_err::{File, OpenOptions};
use nix::sys::stat::Mode;
use nix::unistd::mkfifo;
use regex::bytes::Regex;
use serde_json::{json, Value};
use std::ffi::OsString;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use tempfile::TempDir;

struct OvmfPaths {
    code: PathBuf,
    vars: PathBuf,
    vars_read_only: bool,
}

impl OvmfPaths {
    fn from_dir(dir: &Path, arch: UefiArch) -> Self {
        match arch {
            UefiArch::AArch64 => Self {
                code: dir.join("QEMU_EFI-pflash.raw"),
                vars: dir.join("vars-template-pflash.raw"),
                // The OVMF implementation for AArch64 won't boot unless
                // the vars file is writeable.
                vars_read_only: false,
            },
            UefiArch::IA32 => Self {
                code: dir.join("OVMF32_CODE.fd"),
                vars: dir.join("OVMF32_VARS.fd"),
                vars_read_only: true,
            },
            UefiArch::X86_64 => Self {
                code: dir.join("OVMF_CODE.fd"),
                vars: dir.join("OVMF_VARS.fd"),
                vars_read_only: true,
            },
        }
    }

    fn exists(&self) -> bool {
        self.code.exists() && self.vars.exists()
    }

    /// Find path to OVMF files.
    fn find(opt: &QemuOpt, arch: UefiArch) -> Result<Self> {
        // If the path is specified in the settings, use it.
        if let Some(ovmf_dir) = &opt.ovmf_dir {
            let ovmf_paths = Self::from_dir(ovmf_dir, arch);
            if ovmf_paths.exists() {
                return Ok(ovmf_paths);
            }
            bail!("OVMF files not found in {}", ovmf_dir.display());
        }

        // Check whether the test runner directory contains the files.
        let ovmf_dir = Path::new("uefi-test-runner");
        let ovmf_paths = Self::from_dir(ovmf_dir, arch);
        if ovmf_paths.exists() {
            return Ok(ovmf_paths);
        }

        #[cfg(target_os = "linux")]
        {
            let possible_paths = [
                // Most distros, including CentOS, Fedora, Debian, and Ubuntu.
                Path::new("/usr/share/OVMF"),
                // Arch Linux.
                Path::new("/usr/share/ovmf/x64"),
            ];
            for path in possible_paths {
                let ovmf_paths = Self::from_dir(path, arch);
                if ovmf_paths.exists() {
                    return Ok(ovmf_paths);
                }
            }
        }

        bail!("OVMF files not found anywhere");
    }
}

fn add_pflash_args(cmd: &mut Command, file: &Path, read_only: bool) {
    // Build the argument as an OsString to avoid requiring a UTF-8 path.
    let mut arg = OsString::from("if=pflash,format=raw,readonly=");
    arg.push(if read_only { "on" } else { "off" });
    arg.push(",file=");
    arg.push(file);

    cmd.arg("-drive");
    cmd.arg(arg);
}

struct Pipe {
    qemu_arg: String,
    input_path: PathBuf,
    output_path: PathBuf,
}

impl Pipe {
    fn new(dir: &Path, base_name: &'static str) -> Result<Self> {
        let mode = Mode::from_bits(0o666).unwrap();
        let qemu_arg = format!("pipe:{}", dir.join(base_name).to_str().unwrap());
        let input_path = dir.join(format!("{}.in", base_name));
        let output_path = dir.join(format!("{}.out", base_name));

        mkfifo(&input_path, mode)?;
        mkfifo(&output_path, mode)?;

        Ok(Self {
            qemu_arg,
            input_path,
            output_path,
        })
    }
}

struct Io<R: Read, W: Write> {
    reader: BufReader<R>,
    writer: W,
}

impl<R: Read, W: Write> Io<R, W> {
    fn new(r: R, w: W) -> Self {
        Self {
            reader: BufReader::new(r),
            writer: w,
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

    fn write_line(&mut self, line: &str) -> Result<()> {
        writeln!(self.writer, "{}", line)?;
        self.writer.flush()?;
        Ok(())
    }

    fn write_json(&mut self, json: Value) -> Result<()> {
        self.write_line(&json.to_string())
    }
}

impl Io<File, File> {
    fn from_pipe(pipe: &Pipe) -> Result<Self> {
        Ok(Self::new(
            File::open(&pipe.output_path)?,
            OpenOptions::new().write(true).open(&pipe.input_path)?,
        ))
    }
}

fn echo_filtered_stdout<R: Read, W: Write>(mut child_io: Io<R, W>) {
    // This regex is used to detect and strip ANSI escape codes. These
    // escapes are added by the console output protocol when writing to
    // the serial device.
    let ansi_escape = Regex::new(r"(\x9b|\x1b\[)[0-?]*[ -/]*[@-~]").expect("invalid regex");

    while let Ok(line) = child_io.read_line() {
        let line = line.trim();
        let stripped = ansi_escape.replace_all(line.as_bytes(), &b""[..]);
        let stripped = String::from_utf8(stripped.into()).expect("line is not utf8");

        // Print out the processed QEMU output for logging & inspection.
        println!("{}", stripped);
    }
}

fn process_qemu_io<R: Read, W: Write>(
    mut monitor_io: Io<R, W>,
    mut serial_io: Io<R, W>,
    tmp_dir: &Path,
) -> Result<()> {
    // Execute the QEMU monitor handshake, doing basic sanity checks.
    assert!(monitor_io.read_line()?.starts_with(r#"{"QMP":"#));
    monitor_io.write_json(json!({"execute": "qmp_capabilities"}))?;
    assert_eq!(monitor_io.read_json()?, json!({"return": {}}));

    while let Ok(line) = serial_io.read_line() {
        // Strip whitespace from the end. No need to strip ANSI escape
        // codes like in the stdout, because those escape codes are
        // inserted by the console output protocol, whereas the
        // "SCREENSHOT" line we are interested in is written via the
        // serial protocol.
        let line = line.trim_end();

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
            serial_io.write_line("OK")?;

            // Compare screenshot to the reference file specified by the user.
            // TODO: Add an operating mode where the reference is created if it doesn't exist.
            let reference_file =
                Path::new("uefi-test-runner/screenshots").join(format!("{}.ppm", reference_name));
            let expected = fs_err::read(reference_file)?;
            let actual = fs_err::read(&screenshot_path)?;
            assert_eq!(expected, actual);
        }
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
    let built_file = build_dir.join("uefi-test-runner.efi");
    let output_file = match *opt.target {
        UefiArch::AArch64 => "BootAA64.efi",
        UefiArch::IA32 => "BootIA32.efi",
        UefiArch::X86_64 => "BootX64.efi",
    };
    if !boot_dir.exists() {
        fs_err::create_dir_all(&boot_dir)?;
    }
    fs_err::copy(built_file, boot_dir.join(output_file))?;
    Ok(esp_dir)
}

pub fn run_qemu(arch: UefiArch, opt: &QemuOpt) -> Result<()> {
    let esp_dir = build_esp_dir(opt)?;

    let qemu_exe = match arch {
        UefiArch::AArch64 => "qemu-system-aarch64",
        UefiArch::IA32 => "qemu-system-i386",
        UefiArch::X86_64 => "qemu-system-x86_64",
    };
    let mut cmd = Command::new(qemu_exe);

    // Disable default devices.
    // QEMU by defaults enables a ton of devices which slow down boot.
    cmd.arg("-nodefaults");

    match arch {
        UefiArch::AArch64 => {
            // Use a generic ARM environment. Sadly qemu can't emulate a
            // RPi 4 like machine though.
            cmd.args(&["-machine", "virt"]);

            // A72 is a very generic 64-bit ARM CPU in the wild.
            cmd.args(&["-cpu", "cortex-a72"]);
        }
        UefiArch::IA32 => {}
        UefiArch::X86_64 => {
            // Use a modern machine.
            cmd.args(&["-machine", "q35"]);

            // Multi-processor services protocol test needs exactly 4 CPUs.
            cmd.args(&["-smp", "4"]);

            // Allocate some memory.
            cmd.args(&["-m", "256M"]);

            // Enable hardware-accelerated virtualization if possible.
            if !opt.disable_kvm && !opt.ci {
                cmd.arg("--enable-kvm");
            }

            // Exit instead of rebooting in the CI.
            if opt.ci {
                cmd.arg("-no-reboot");
            }

            // Map the QEMU exit signal to port f4.
            cmd.args(&["-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]);

            // OVMF debug builds can output information to a serial `debugcon`.
            // Only enable when debugging UEFI boot.
            // cmd.args(&[
            //     "-debugcon",
            //     "file:debug.log",
            //     "-global",
            //     "isa-debugcon.iobase=0x402",
            // ]);
        }
    }

    // Set up OVMF.
    let ovmf_paths = OvmfPaths::find(opt, arch)?;
    add_pflash_args(&mut cmd, &ovmf_paths.code, /*read_only=*/ true);
    add_pflash_args(&mut cmd, &ovmf_paths.vars, ovmf_paths.vars_read_only);

    // Mount a local directory as a FAT partition.
    cmd.arg("-drive");
    let mut drive_arg = OsString::from("format=raw,file=fat:rw:");
    drive_arg.push(esp_dir);
    cmd.arg(drive_arg);

    // When running in headless mode we don't have video, but we can still have
    // QEMU emulate a display and take screenshots from it.
    cmd.args(&["-vga", "std"]);
    if opt.headless {
        cmd.args(&["-display", "none"]);
    }

    let tmp_dir = TempDir::new()?;
    let tmp_dir = tmp_dir.path();

    let qemu_monitor_pipe = Pipe::new(tmp_dir, "qemu-monitor")?;
    let serial_pipe = Pipe::new(tmp_dir, "serial")?;

    // Open two serial devices. The first one is connected to the host's
    // stdout, and serves to just transport logs. The second one is
    // connected to a pipe, and used to receive the SCREENSHOT command
    // and send the response. That second will also receive logs up
    // until the test runner opens the handle in exclusive mode, but we
    // can just read and ignore those lines.
    cmd.args(&["-serial", "stdio"]);
    cmd.args(&["-serial", &serial_pipe.qemu_arg]);

    // Map the QEMU monitor to a pair of named pipes
    cmd.args(&["-qmp", &qemu_monitor_pipe.qemu_arg]);

    println!("{}", command_to_string(&cmd));

    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    let mut child = cmd.spawn()?;

    let monitor_io = Io::from_pipe(&qemu_monitor_pipe)?;
    let serial_io = Io::from_pipe(&serial_pipe)?;
    let child_io = Io::new(child.stdout.take().unwrap(), child.stdin.take().unwrap());

    // Start a thread to process stdout from the child.
    let stdout_thread = thread::spawn(|| echo_filtered_stdout(child_io));

    // Capture the result to return it, but first wait for the child to
    // exit.
    let ret = process_qemu_io(monitor_io, serial_io, tmp_dir);
    child.wait()?;

    stdout_thread.join().expect("stdout thread panicked");

    ret
}
