#!/usr/bin/env python3

'Script used to build, run, and test the code on all supported platforms.'

import argparse
import os
from pathlib import Path
import shutil
import subprocess as sp
import sys

from warnings import warn

## Configurable settings
# Target to build for.
TARGET = 'x86_64-uefi'
# Configuration to build.
CONFIG = 'debug'

# Xargo executable.
XARGO = 'xargo'

# QEMU executable to use
QEMU = 'qemu-system-x86_64'

# Path to workspace directory (which contains the top-level `Cargo.toml`)
WORKSPACE_DIR = Path(__file__).resolve().parents[1]

# Path to directory containing `OVMF_{CODE/VARS}.fd`.
# TODO: use installed OVMF, if available.
OVMF_DIR = WORKSPACE_DIR / 'uefi-test-runner'

BUILD_DIR = WORKSPACE_DIR / 'target' / TARGET / CONFIG
ESP_DIR = BUILD_DIR / 'esp'

def run_xargo(verb, *flags):
    'Runs Xargo with certain arguments.'
    cmd_line = [XARGO, verb, '--target', TARGET, *flags]
    print(' '.join(cmd_line))
    sp.run(cmd_line).check_returncode()

def build():
    'Builds the tests package.'

    run_xargo('build', '--package', 'uefi-test-runner')

    # Copy the built file to the right directory for running tests.
    built_file = BUILD_DIR / 'uefi-test-runner.efi'

    boot_dir = ESP_DIR / 'EFI' / 'Boot'
    boot_dir.mkdir(parents=True, exist_ok=True)

    output_file = boot_dir / 'BootX64.efi'

    shutil.copy2(built_file, output_file)

def doc():
    'Generates documentation for the main crate.'
    run_xargo('doc', '--no-deps', '--package', 'uefi')

def clippy():
    'Analyses the code with Clippy.'
    run_xargo('clippy')

def run_qemu():
    'Runs the code in QEMU.'

    # Ask xargo to rebuild changes.
    build()

    firmware_files = [];

    for file in os.listdir(OVMF_DIR):
        filename = os.fsdecode(file)
        if filename.endswith(".fd"):
            new_path = os.path.join(filename)
            print('using firmware file ' + new_path)
            firmware_files.append(new_path)
    if(len(firmware_files) < 1):
        warn('found no firmware in ' + str(OVMF_DIR))

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
        #'-drive', f'if=pflash,format=raw,file={ovmf_code},readonly=on',
        #'-drive', f'if=pflash,format=raw,file={ovmf_vars},readonly=on',

        # Create AHCI controller.
        '-device', 'ahci,id=ahci,multifunction=on',

        # Mount a local directory as a FAT partition.
        '-drive', f'if=none,format=raw,file=fat:rw:{ESP_DIR},id=esp',
        '-device', 'ide-drive,bus=ahci.0,drive=esp',

        # OVMF debug builds can output information to a serial `debugcon`.
        # Only enable when debugging UEFI boot:
        #'-debugcon', 'file:debug.log', '-global', 'isa-debugcon.iobase=0x402',
    ]

    # Set up OVMF.
    for path in firmware_files:
        qemu_flags.append('-drive')
        qemu_flags.append(f'if=pflash,format=raw,file={path},readonly=on')

    print(' '.join([QEMU] + qemu_flags))
    sp.run([QEMU] + qemu_flags).check_returncode()

def main():
    'Runs the user-requested actions.'

    # Currently, Clang fails to build `compiler-builtins`
    if os.environ['CC'] == 'clang':
        os.environ['CC'] = 'gcc'

    # Clear any Rust flags which might affect the build.
    os.environ['RUSTFLAGS'] = ''

    # Temporary solution for https://github.com/rust-lang/cargo/issues/4905
    os.environ['RUST_TARGET_PATH'] = str(WORKSPACE_DIR / 'uefi-test-runner')

    usage = '%(prog)s verb [options]'
    desc = 'Build script for UEFI programs'

    parser = argparse.ArgumentParser(usage=usage, description=desc)

    subparsers = parser.add_subparsers(dest='verb')

    build_parser = subparsers.add_parser('build')
    run_parser = subparsers.add_parser('run')
    doc_parser = subparsers.add_parser('doc')
    clippy_parser = subparsers.add_parser('clippy')

    opts = parser.parse_args()

    if opts.verb == 'build':
        build()
    elif opts.verb == 'run':
        run_qemu()
    elif opts.verb == 'doc':
        doc()
    elif opts.verb == 'clippy':
        clippy()
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
