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

## Configurable settings
# Path to workspace directory (which contains the top-level `Cargo.toml`)
WORKSPACE_DIR = Path(__file__).resolve().parents[1]

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
    # Custom targets need to be given by relative path, instead of only by name
    # We need to append a `.json` to turn the triple into a path
    if SETTINGS['arch'] == 'aarch64':
        target += '.json'

    cmd = ['cargo', tool, '--target', target, *flags]

    if SETTINGS['verbose']:
        print(' '.join(cmd))

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
        build_args.extend(['--features', 'ci'])

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

    run_clippy('--all')

def doc():
    'Generates documentation for the library crates.'
    sp.run([
        'cargo', 'doc', '--no-deps',
        '--package', 'uefi',
        '--package', 'uefi-macros',
        '--package', 'uefi-services',
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

def run_qemu():
    'Runs the code in QEMU.'

    # Rebuild all the changes.
    build('--features', 'qemu')

    ovmf_code, ovmf_vars = ovmf_files(find_ovmf())

    qemu_monitor_pipe = 'qemu-monitor'

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
            # Multi-processor services protocol test needs exactly 3 CPUs.
            '-smp', '3',

            # Allocate some memory.
            '-m', '128M',
        ])
        if not SETTINGS['ci']:
            # Enable acceleration if possible.
            qemu_flags.append('--enable-kvm')
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

        # Connect the serial port to the host. OVMF is kind enough to connect
        # the UEFI stdout and stdin to that port too.
        '-serial', 'stdio',

        # Map the QEMU monitor to a pair of named pipes
        '-qmp', f'pipe:{qemu_monitor_pipe}',
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

    # This regex can be used to detect and strip ANSI escape codes when
    # analyzing the output of the test runner.
    ansi_escape = re.compile(r'(\x9B|\x1B\[)[0-?]*[ -/]*[@-~]')

    # Setup named pipes as a communication channel with QEMU's monitor
    monitor_input_path = f'{qemu_monitor_pipe}.in'
    os.mkfifo(monitor_input_path)
    monitor_output_path = f'{qemu_monitor_pipe}.out'
    os.mkfifo(monitor_output_path)

    # Start QEMU
    qemu = sp.Popen(cmd, stdin=sp.PIPE, stdout=sp.PIPE, universal_newlines=True)
    try:
        # Connect to the QEMU monitor
        with open(monitor_input_path, mode='w') as monitor_input,                  \
             open(monitor_output_path, mode='r') as monitor_output:
            # Execute the QEMU monitor handshake, doing basic sanity checks
            assert monitor_output.readline().startswith('{"QMP":')
            print('{"execute": "qmp_capabilities"}', file=monitor_input, flush=True)
            assert monitor_output.readline() == '{"return": {}}\n'

            # Iterate over stdout...
            for line in qemu.stdout:
                # Strip ending and trailing whitespace + ANSI escape codes
                # (This simplifies log analysis and keeps the terminal clean)
                stripped = ansi_escape.sub('', line.strip())

                # Skip lines which contain nothing else
                if not stripped:
                    continue

                # Print out the processed QEMU output for logging & inspection
                print(stripped)

                # If the app requests a screenshot, take it
                if stripped.startswith("SCREENSHOT: "):
                    reference_name = stripped[12:]

                    # Ask QEMU to take a screenshot
                    monitor_command = '{"execute": "screendump", "arguments": {"filename": "screenshot.ppm"}}'
                    print(monitor_command, file=monitor_input, flush=True)

                    # Wait for QEMU's acknowledgement, ignoring events
                    reply = json.loads(monitor_output.readline())
                    while "event" in reply:
                        reply = json.loads(monitor_output.readline())
                    assert reply == {"return": {}}

                    # Tell the VM that the screenshot was taken
                    print('OK', file=qemu.stdin, flush=True)

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

        # Delete the monitor pipes
        os.remove(monitor_input_path)
        os.remove(monitor_output_path)

        # Throw an exception if QEMU failed
        if status != 0:
            raise sp.CalledProcessError(cmd=cmd, returncode=status)

def main():
    'Runs the user-requested actions.'

    # Clear any Rust flags which might affect the build.
    os.environ['RUSTFLAGS'] = ''

    desc = 'Build script for UEFI programs'

    parser = argparse.ArgumentParser(description=desc)

    parser.add_argument('verb', help='command to run', type=str,
                        choices=['build', 'run', 'doc', 'clippy'])

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

    opts = parser.parse_args()

    SETTINGS['arch'] = opts.target
    # Check if we need to enable verbose mode
    SETTINGS['verbose'] = opts.verbose
    SETTINGS['headless'] = opts.headless
    SETTINGS['config'] = 'release' if opts.release else 'debug'
    SETTINGS['ci'] = opts.ci

    verb = opts.verb

    if verb == 'build':
        build()
    elif verb == 'clippy':
        clippy()
    elif verb == 'doc':
        doc()
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
