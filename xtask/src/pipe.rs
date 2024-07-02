use crate::platform;
use crate::qemu::Io;
use anyhow::Result;
use fs_err::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

pub struct Pipe {
    qemu_arg: String,
    input_path: PathBuf,
    output_path: PathBuf,
}

impl Pipe {
    /// Prepare to set up a two-way communication pipe. This is called
    /// before launching QEMU. On Unix this uses `mkfifo` to create two
    /// pipes; on Windows QEMU itself will create the duplex pipe.
    pub fn new(dir: &Path, base_name: &'static str) -> Result<Self> {
        if platform::is_unix() {
            let qemu_arg = format!("pipe:{}", dir.join(base_name).to_str().unwrap());
            let input_path = dir.join(format!("{}.in", base_name));
            let output_path = dir.join(format!("{}.out", base_name));

            // This part has to be conditionally compiled because the
            // `nix` interfaces don't exist when compiling under
            // Windows.
            #[cfg(unix)]
            {
                let mode = nix::sys::stat::Mode::from_bits(0o666).unwrap();
                nix::unistd::mkfifo(&input_path, mode)?;
                nix::unistd::mkfifo(&output_path, mode)?;
            }

            Ok(Self {
                qemu_arg,
                input_path,
                output_path,
            })
        } else if platform::is_windows() {
            // No need to call the equivalent of `mkfifo` here; QEMU acts as
            // the named pipe server and creates the pipe itself.

            Ok(Self {
                // QEMU adds the "\\.\pipe\" prefix automatically.
                qemu_arg: format!("pipe:{}", base_name),

                // On Windows the pipe is duplex, so only one path
                // needed.
                input_path: format!(r"\\.\pipe\{}", base_name).into(),
                output_path: PathBuf::new(),
            })
        } else {
            unimplemented!();
        }
    }

    pub fn qemu_arg(&self) -> &str {
        &self.qemu_arg
    }

    /// Create an `Io` object for performing reads and writes.
    pub fn open_io(&self) -> Result<Io<File, File>> {
        let reader;
        let writer;

        if platform::is_unix() {
            reader = File::open(&self.output_path)?;
            writer = OpenOptions::new().write(true).open(&self.input_path)?;
        } else if platform::is_windows() {
            // Connect to the pipe, then clone the resulting `File` so
            // that we can wrap the read side in a `BufReader`. The
            // reader and writer must share the same underlying
            // `Handle`, so this is different than opening the pipe
            // twice.
            writer = windows_open_pipe(&self.input_path)?;
            reader = writer.try_clone()?;
        } else {
            unimplemented!();
        }

        Ok(Io::new(reader, writer))
    }
}

/// Attempt to connect to a duplex named pipe in byte mode.
fn windows_open_pipe(path: &Path) -> Result<File> {
    let max_attempts = 100;
    let mut attempt = 0;
    loop {
        attempt += 1;

        match OpenOptions::new().read(true).write(true).open(path) {
            Ok(file) => return Ok(file),
            Err(err) => {
                if attempt >= max_attempts {
                    return Err(err)?;
                } else {
                    // Sleep before trying again.
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }
}
