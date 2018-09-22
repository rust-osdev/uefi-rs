#!/usr/bin/env python3

'Script used to build, run, and test the code on all supported platforms.'

import argparse
import os
from pathlib import Path
import shutil
import subprocess as sp
import sys

## Configurable settings
# Target to build for.
TARGET = 'x86_64-uefi'
# Configuration to build.
CONFIG = 'debug'

# QEMU executable to use
QEMU = 'qemu-system-x86_64'

# Path to workspace directory (which contains the top-level `Cargo.toml`)
WORKSPACE_DIR = Path(__file__).resolve().parents[1]

# Path to directory containing `OVMF_{CODE/VARS}.fd`.
# TODO: use installed OVMF, if available.
OVMF_DIR = WORKSPACE_DIR / 'uefi-test-runner'

# Set to `True` or use the `--verbose` argument to print commands.
VERBOSE = False

BUILD_DIR = WORKSPACE_DIR / 'target' / TARGET / CONFIG
ESP_DIR = BUILD_DIR / 'esp'

def run_xbuild(*flags):
    'Runs Cargo XBuild with certain arguments.'

    cmd = ['cargo', 'xbuild', '--target', TARGET, *flags]

    if VERBOSE:
        print(' '.join(cmd))

    sp.run(cmd).check_returncode()

def build(*test_flags):
    'Builds the tests and examples.'

    run_xbuild('--package', 'uefi-test-runner', *test_flags)
    run_xbuild('--package', 'uefi', '--examples')

    # Copy the built test runner file to the right directory for running tests.
    built_file = BUILD_DIR / 'uefi-test-runner.efi'

    boot_dir = ESP_DIR / 'EFI' / 'Boot'
    boot_dir.mkdir(parents=True, exist_ok=True)

    output_file = boot_dir / 'BootX64.efi'

    shutil.copy2(built_file, output_file)

def doc():
    'Generates documentation for the library crates.'
    sp.run(
        'cargo', 'doc', '--no-deps',
        '--package', 'uefi',
        '--package', 'uefi-utils',
        '--package', 'uefi-alloc',
        '--package', 'uefi-logger',
        '--package', 'uefi-services',
    )

def run_qemu():
    'Runs the code in QEMU.'

    # Rebuild all the changes.
    build('--features', 'qemu-f4-exit')

    ovmf_code, ovmf_vars = OVMF_DIR / 'OVMF_CODE.fd', OVMF_DIR / 'OVMF_VARS.fd'

    if not ovmf_code.is_file():
        raise FileNotFoundError(f'OVMF_CODE.fd not found in the `{OVMF_DIR}` directory')

    examples_dir = BUILD_DIR / 'examples'

    qemu_flags = [
        # Disable default devices.
        # QEMU by defaults enables a ton of devices which slow down boot.
        '-nodefaults',

        # Use a standard VGA for graphics.
        '-vga', 'std',

        # Use a modern machine, with acceleration if possible.
        '-machine', 'q35,accel=kvm:tcg',

        # Allocate some memory.
        '-m', '128M',

        # Set up OVMF.
        '-drive', f'if=pflash,format=raw,file={ovmf_code},readonly=on',
        '-drive', f'if=pflash,format=raw,file={ovmf_vars},readonly=on',

        # Mount a local directory as a FAT partition.
        '-drive', f'format=raw,file=fat:rw:{ESP_DIR}',

        # Mount the built examples directory.
        '-drive', f'format=raw,file=fat:rw:{examples_dir}',

        # Map the QEMU exit signal to port f4
        '-device', 'isa-debug-exit,iobase=0xf4,iosize=0x04',

        # Add a null serial device for testing with loop-back
        '-serial', 'null',

        # OVMF debug builds can output information to a serial `debugcon`.
        # Only enable when debugging UEFI boot:
        #'-debugcon', 'file:debug.log', '-global', 'isa-debugcon.iobase=0x402',
    ]

    cmd = [QEMU] + qemu_flags

    if VERBOSE:
        print(' '.join(cmd))

    sp.run(cmd).check_returncode()

def main():
    'Runs the user-requested actions.'

    # Clear any Rust flags which might affect the build.
    os.environ['RUSTFLAGS'] = ''

    os.environ['RUST_TARGET_PATH'] = str(WORKSPACE_DIR / 'uefi-test-runner')

    usage = '%(prog)s verb [options]'
    desc = 'Build script for UEFI programs'

    parser = argparse.ArgumentParser(usage=usage, description=desc)

    parser.add_argument('--verbose', '-v', action='store_true')

    subparsers = parser.add_subparsers(dest='verb')

    build_parser = subparsers.add_parser('build')
    run_parser = subparsers.add_parser('run')
    doc_parser = subparsers.add_parser('doc')

    opts = parser.parse_args()

    # Check if we need to enable verbose mode
    global VERBOSE
    VERBOSE = VERBOSE or opts.verbose

    if opts.verb == 'build':
        build()
    elif opts.verb == 'run':
        run_qemu()
    elif opts.verb == 'doc':
        doc()
    elif opts.verb is None or opts.verb == '':
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
