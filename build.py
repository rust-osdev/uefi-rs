#!/usr/bin/env python3

'Script used to build, run, and test the code on all supported platforms.'

import argparse
import filecmp
import json
import os
from pathlib import Path
import re
import shutil
import subprocess as sp
import sys
import threading

## Configurable settings
# Path to workspace directory (which contains the top-level `Cargo.toml`)
WORKSPACE_DIR = Path(__file__).resolve().parent

# Try changing these with command line flags, where possible
SETTINGS = {
    # Architecture to build for
    'arch': 'x86_64',
    # Print commands before running them.
    'verbose': False,
    # Run QEMU without showing GUI
    'headless': False,
    # Configuration to build.
    'config': 'debug',
    # Disables some tests which don't work in our CI setup
    'ci': False,
    # KVM is a Linux kernel module which allows QEMU to use
    # hardware-accelerated virtualization.
    'disable_kvm': False,
    # QEMU executable to use
    # Indexed by the `arch` setting
    'qemu_binary': {
        'x86_64': 'qemu-system-x86_64',
        'aarch64': 'qemu-system-aarch64',
    },
    # Path to directory containing `OVMF_{CODE/VARS}.fd` (for x86_64),
    # or `*-pflash.raw` (for AArch64).
    # `find_ovmf` function will try to find one if this isn't specified.
    'ovmf_dir': None,
}

# Path to target directory. If None, it will be initialized with information
# from cargo metadata at the first time target_dir function is invoked.
TARGET_DIR = None

def target_dir():
    'Returns the target directory'
    global TARGET_DIR
    if TARGET_DIR is None:
        cmd = ['cargo', 'metadata', '--format-version=1']
        result = sp.run(cmd, stdout=sp.PIPE, check=True)
        TARGET_DIR = Path(json.loads(result.stdout)['target_directory'])
    return TARGET_DIR

def get_target_triple():
    arch = SETTINGS['arch']
    return f'{arch}-unknown-uefi'

def build_dir():
    'Returns the directory where Cargo places the build artifacts'
    return target_dir() / get_target_triple() / SETTINGS['config']

def esp_dir():
    'Returns the directory where we will build the emulated UEFI system partition'
    return build_dir() / 'esp'

def run_tool(tool, *flags):
    'Runs cargo-<tool> with certain arguments.'

    target = get_target_triple()
    cmd = ['cargo', tool, '--target', target, *flags]

    if SETTINGS['verbose']:
        print(' '.join(str(arg) for arg in cmd))

    sp.run(cmd, check=True)

def run_build(*flags):
    'Runs cargo-build with certain arguments.'
    run_tool('build', *flags)

def run_clippy(*flags):
    'Runs cargo-clippy with certain arguments.'
    run_tool('clippy', *flags)

def build(*test_flags):
    'Builds the test crate.'

    build_args = [
        '--package', 'uefi-test-runner',
        *test_flags,
    ]

    if SETTINGS['config'] == 'release':
        build_args.append('--release')

    if SETTINGS['ci']:
        build_args.extend(['--features', 'uefi-test-runner/ci'])

    run_build(*build_args)

    # Copy the built test runner file to the right directory for running tests.
    built_file = build_dir() / 'uefi-test-runner.efi'

    boot_dir = esp_dir() / 'EFI' / 'Boot'
    boot_dir.mkdir(parents=True, exist_ok=True)

    arch = SETTINGS['arch']
    if arch == 'x86_64':
        output_file = boot_dir / 'BootX64.efi'
    elif arch == 'aarch64':
        output_file = boot_dir / 'BootAA64.efi'

    shutil.copy2(built_file, output_file)

def clippy():
    'Runs Clippy on all projects'

    run_clippy(
        # Specifying the manifest path allows this command to
        # run successfully regardless of the CWD.
        '--manifest-path', WORKSPACE_DIR / 'Cargo.toml',
        # Lint all packages in the workspace.
        '--workspace',
        # Enable all the features in the uefi package that enable more
        # code.
        '--features=alloc,exts,logger',
        # Treat all warnings as errors.
        '--', '-D', 'warnings')

def doc():
    'Generates documentation for the library crates.'
    sp.run([
        'cargo', 'doc', '--no-deps',
        '--package', 'uefi',
        '--package', 'uefi-macros',
        '--package', 'uefi-services',
    ], check=True)

def get_rustc_cfg():
    'Run and parse "rustc --print=cfg" as key, val pairs.'
    output = sp.run([
        'rustc', '--print=cfg'
    ], check=True, capture_output=True, text=True).stdout
    for line in output.splitlines():
        parts = line.split('=', maxsplit=1)
        # Only interested in the lines that look like this: key="val"
        if len(parts) == 2:
            key = parts[0]
            val = parts[1]
            # Strip the quotes
            if val.startswith('"') and val.endswith('"'):
                val = val[1:-1]
            yield key, val

def get_host_target():
    'Get the host target, e.g. "x86_64-unknown-linux-gnu".'
    cfg = dict(get_rustc_cfg())
    arch = cfg['target_arch']
    vendor = cfg['target_vendor']
    os = cfg['target_os']
    env = cfg['target_env']
    return f'{arch}-{vendor}-{os}-{env}'

def test():
    'Run tests and doctests using the host target.'
    sp.run([
        'cargo', 'test',
        # Specifying the manifest path allows this command to
        # run successfully regardless of the CWD.
        '--manifest-path', WORKSPACE_DIR / 'Cargo.toml',
        '-Zbuild-std=std',
        '--target', get_host_target(),
        '--features', 'exts',
        '--package', 'uefi',
        '--package', 'uefi-macros',
        # Don't test uefi-services (or the packages that depend on it)
        # as it has lang items that conflict with `std`.
    ], check=True)

def ovmf_files(ovmf_dir):
    'Returns the tuple of paths to the OVMF code and vars firmware files, given the directory'
    if SETTINGS['arch'] == 'x86_64':
        return ovmf_dir / 'OVMF_CODE.fd', ovmf_dir / 'OVMF_VARS.fd'
    if SETTINGS['arch'] == 'aarch64':
        return ovmf_dir / 'QEMU_EFI-pflash.raw', ovmf_dir / 'vars-template-pflash.raw'
    raise NotImplementedError('Target arch not supported')

def check_ovmf_dir(ovmf_dir):
    'Check whether the given directory contains necessary OVMF files'
    ovmf_code, ovmf_vars = ovmf_files(ovmf_dir)
    return ovmf_code.is_file() and ovmf_vars.is_file()

def find_ovmf():
    'Find path to OVMF files'

    # If the path is specified in the settings, use it.
    if SETTINGS['ovmf_dir'] is not None:
        ovmf_dir = SETTINGS['ovmf_dir']
        if check_ovmf_dir(ovmf_dir):
            return ovmf_dir
        raise FileNotFoundError(f'OVMF files not found in `{ovmf_dir}`')

    # Check whether the test runner directory contains the files.
    ovmf_dir = WORKSPACE_DIR / 'uefi-test-runner'
    if check_ovmf_dir(ovmf_dir):
        return ovmf_dir

    if sys.platform.startswith('linux'):
        possible_paths = [
            # Most distros, including CentOS, Fedora, Debian, and Ubuntu.
            Path('/usr/share/OVMF'),
            # Arch Linux
            Path('/usr/share/ovmf/x64'),
        ]
        for path in possible_paths:
            if check_ovmf_dir(path):
                return path

    raise FileNotFoundError(f'OVMF files not found anywhere')

def echo_filtered_stdout(stdout):
    """Print lines read from the QEMU process's stdout."""
    # This regex is used to detect and strip ANSI escape codes. These
    # escapes are added by the console output protocol when writing to
    # the serial device.
    ansi_escape = re.compile(r'(\x9B|\x1B\[)[0-?]*[ -/]*[@-~]')

    for line in stdout:
        # Print out the processed QEMU output for logging & inspection.
        # Strip ending and trailing whitespace + ANSI escape codes
        # (This simplifies log analysis and keeps the terminal clean)
        print(ansi_escape.sub('', line.strip()))

class Pipe:
    def __init__(self, base_name):
        self.qemu_arg = f'pipe:{base_name}'
        self.input_path = f'{base_name}.in'
        self.output_path = f'{base_name}.out'

        os.mkfifo(self.input_path)
        os.mkfifo(self.output_path)

    def remove_files(self):
        os.remove(self.input_path)
        os.remove(self.output_path)

def run_qemu():
    'Runs the code in QEMU.'

    # Rebuild all the changes.
    build('--features', 'uefi-test-runner/qemu')

    ovmf_code, ovmf_vars = ovmf_files(find_ovmf())

    # Set up named pipes as communication channels with QEMU.
    qemu_monitor_pipe = Pipe('qemu-monitor')
    serial_pipe = Pipe('serial-pipe')

    arch = SETTINGS['arch']

    qemu_flags = [
        # Disable default devices.
        # QEMU by defaults enables a ton of devices which slow down boot.
        '-nodefaults',
    ]

    ovmf_vars_readonly = 'on'
    if arch == 'aarch64':
        # The OVMF implementation for AArch64 won't boot unless the
        # vars file is writeable.
        ovmf_vars_readonly = 'off'

    if arch == 'x86_64':
        qemu_flags.extend([
            # Use a modern machine,.
            '-machine', 'q35',

            # Multi-processor services protocol test needs exactly 4 CPUs.
            '-smp', '4',

            # Allocate some memory.
            '-m', '256M',
        ])
        if not SETTINGS['ci']:
            # Enable hardware-accelerated virtualization if possible.
            if not SETTINGS['disable_kvm']:
                qemu_flags.append('--enable-kvm')
        else:
            # Exit instead of rebooting
            qemu_flags.append('-no-reboot')
    elif arch == 'aarch64':
        qemu_flags.extend([
            # Use a generic ARM environment. Sadly qemu can't emulate a RPi 4 like machine though
            '-machine', 'virt',

            # A72 is a very generic 64-bit ARM CPU in the wild
            '-cpu', 'cortex-a72',
        ])
    else:
        raise NotImplementedError('Unknown arch')

    qemu_flags.extend([
        # Set up OVMF.
        '-drive', f'if=pflash,format=raw,file={ovmf_code},readonly=on',
        '-drive', f'if=pflash,format=raw,file={ovmf_vars},readonly={ovmf_vars_readonly}',

        # Mount a local directory as a FAT partition.
        '-drive', f'format=raw,file=fat:rw:{esp_dir()}',

        # Open two serial devices. The first one is connected to the
        # host's stdout, and serves to just transport logs. The second
        # one is connected to a pipe, and used to receive the SCREENSHOT
        # command and send the response. That second will also receive
        # logs up until the test runner opens the handle in exclusive
        # mode, but we can just read and ignore those lines.
        '-serial', 'stdio',
        '-serial', serial_pipe.qemu_arg,

        # Map the QEMU monitor to a pair of named pipes
        '-qmp', qemu_monitor_pipe.qemu_arg,
    ])

    # For now these only work on x86_64
    if arch == 'x86_64':
        # Enable debug features
        qemu_flags.extend([
            # Map the QEMU exit signal to port f4
            '-device', 'isa-debug-exit,iobase=0xf4,iosize=0x04',

            # OVMF debug builds can output information to a serial `debugcon`.
            # Only enable when debugging UEFI boot:
            #'-debugcon', 'file:debug.log', '-global', 'isa-debugcon.iobase=0x402',
        ])

    # When running in headless mode we don't have video, but we can still have
    # QEMU emulate a display and take screenshots from it.
    qemu_flags.extend(['-vga', 'std'])
    if SETTINGS['headless']:
        # Do not attach a window to QEMU's display
        qemu_flags.extend(['-display', 'none'])

    qemu_binary = SETTINGS['qemu_binary'][arch]
    cmd = [qemu_binary] + qemu_flags

    if SETTINGS['verbose']:
        print(' '.join(cmd))

    # Start QEMU
    qemu = sp.Popen(cmd, stdout=sp.PIPE, universal_newlines=True)
    thread = threading.Thread(target=echo_filtered_stdout, args=(qemu.stdout,))
    thread.start()
    try:
        # Connect to the QEMU monitor
        with open(qemu_monitor_pipe.input_path, mode='w') as monitor_input,   \
             open(qemu_monitor_pipe.output_path, mode='r') as monitor_output, \
             open(serial_pipe.input_path, mode='w') as serial_input,          \
             open(serial_pipe.output_path, mode='r') as serial_output:
            # Execute the QEMU monitor handshake, doing basic sanity checks
            assert monitor_output.readline().startswith('{"QMP":')
            print('{"execute": "qmp_capabilities"}', file=monitor_input, flush=True)
            assert monitor_output.readline() == '{"return": {}}\n'

            # Iterate over the second serial device's output...
            for line in serial_output:
                # Strip whitespace from the end. No need to strip ANSI
                # escape codes like in the stdout, because those escape
                # codes are inserted by the console output protocol,
                # whereas the "SCREENSHOT" line we are interested in is
                # written via the serial protocol.
                line = line.rstrip()

                # If the app requests a screenshot, take it
                if line.startswith("SCREENSHOT: "):
                    print(line)

                    reference_name = line[12:]

                    # Ask QEMU to take a screenshot
                    monitor_command = '{"execute": "screendump", "arguments": {"filename": "screenshot.ppm"}}'
                    print(monitor_command, file=monitor_input, flush=True)

                    # Wait for QEMU's acknowledgement, ignoring events
                    reply = json.loads(monitor_output.readline())
                    while "event" in reply:
                        reply = json.loads(monitor_output.readline())
                    assert reply == {"return": {}}

                    # Tell the VM that the screenshot was taken
                    print('OK', file=serial_input, flush=True)

                    # Compare screenshot to the reference file specified by the user
                    # TODO: Add an operating mode where the reference is created if it doesn't exist
                    reference_file = WORKSPACE_DIR / 'uefi-test-runner' / 'screenshots' / (reference_name + '.ppm')
                    assert filecmp.cmp('screenshot.ppm', reference_file)

                    # Delete the screenshot once done
                    os.remove('screenshot.ppm')
    finally:
        try:
            # Wait for QEMU to finish
            status = qemu.wait()
        except sp.TimeoutExpired:
            print('Tests are taking too long to run, killing QEMU', file=sys.stderr)
            qemu.kill()
            status = -1

        # Delete the pipes
        qemu_monitor_pipe.remove_files()
        serial_pipe.remove_files()

        # Throw an exception if QEMU failed
        if status != 0 and status != 3:
            raise sp.CalledProcessError(cmd=cmd, returncode=status)

    thread.join()

def main():
    'Runs the user-requested actions.'

    # Clear any Rust flags which might affect the build.
    os.environ['RUSTFLAGS'] = ''

    desc = 'Build script for UEFI programs'

    parser = argparse.ArgumentParser(description=desc)

    parser.add_argument('verb', help='command to run', type=str,
                        choices=['build', 'run', 'doc', 'clippy', 'test'])

    parser.add_argument('--target', help='target to build for (default: %(default)s)', type=str,
                        choices=['x86_64', 'aarch64'], default='x86_64')

    parser.add_argument('--verbose', '-v', help='print commands before executing them',
                        action='store_true')

    parser.add_argument('--headless', help='run QEMU without a GUI',
                        action='store_true')

    parser.add_argument('--release', help='build in release mode',
                        action='store_true')

    parser.add_argument('--ci', help='disables some tests which currently break CI',
                        action='store_true')

    parser.add_argument('--disable-kvm', help='disables hardware accelerated virtualization support in QEMU',
                        action='store_true')

    opts = parser.parse_args()

    SETTINGS['arch'] = opts.target
    # Check if we need to enable verbose mode
    SETTINGS['verbose'] = opts.verbose
    SETTINGS['headless'] = opts.headless
    SETTINGS['config'] = 'release' if opts.release else 'debug'
    SETTINGS['ci'] = opts.ci
    SETTINGS['disable_kvm'] = opts.disable_kvm

    verb = opts.verb

    if verb == 'build':
        build()
    elif verb == 'clippy':
        clippy()
    elif verb == 'doc':
        doc()
    elif verb == 'test':
        test()
    elif verb == 'run' or verb is None or opts.verb == '':
        # Run the program, by default.
        run_qemu()
    else:
        raise ValueError(f'Unknown verb {opts.verb}')

if __name__ == '__main__':
    try:
        main()
    except sp.CalledProcessError as cpe:
        print(f'Subprocess {cpe.cmd[0]} exited with error code {cpe.returncode}')
        sys.exit(1)
