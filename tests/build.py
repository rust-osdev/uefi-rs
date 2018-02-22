#!/usr/bin/env python3

'Script used to build, run, and test the code on all supported platforms.'

import os
from pathlib import Path
import subprocess as sp
import sys

## Configurable settings
# Target to build for.
TARGET = 'x86_64-uefi'
# Configuration to build.
CONFIG = 'debug'

# Xargo executable.
XARGO = 'xargo'

# A linker for PE/COFF files.
LINKER = 'lld'
LINKER_FLAGS = [
    # Use LLD in `link.exe` mode.
    '-flavor', 'link',
    # Create 64-bit executables.
    '/Machine:x64',
    # Create UEFI apps.
    '/Subsystem:EFI_Application',
    # Customizable entry point name.
    '/Entry:uefi_start',
]

# QEMU executable to use
QEMU = 'qemu-system-x86_64'

# Path to workspace directory (which contains the top-level `Cargo.toml`)
WORKSPACE_DIR = Path(__file__).resolve().parents[1]

# Path to directory containing `OVMF_{CODE/VARS}.fd`.
# TODO: use installed OVMF, if available.
OVMF_DIR = WORKSPACE_DIR / 'tests'

BUILD_DIR = WORKSPACE_DIR / 'target' / TARGET / CONFIG
ESP_DIR = BUILD_DIR / 'esp'

# File with test output.
LOG_FILE = BUILD_DIR / 'tests.log'

def run_xargo(verb, *flags):
    'Runs Xargo with certain arguments.'
    sp.run([XARGO, verb, '--target', TARGET, *flags]).check_returncode()

def build():
    'Builds the tests package.'

    run_xargo('build', '--package', 'tests')

    input_lib = BUILD_DIR / 'libtests.a'

    boot_dir = ESP_DIR / 'EFI' / 'Boot'
    boot_dir.mkdir(parents=True, exist_ok=True)

    output = boot_dir / 'BootX64.efi'

    sp.run([LINKER, *LINKER_FLAGS, str(input_lib), f'-Out:{output}']).check_returncode()

def doc():
    'Generates documentation for the main crate.'
    run_xargo('doc', '--no-deps', '--package', 'uefi')

def clippy():
    'Analyses the code with Clippy.'
    run_xargo('clippy')

def run_qemu():
    'Runs the code in QEMU.'
    ovmf_code, ovmf_vars = OVMF_DIR / 'OVMF_CODE.fd', OVMF_DIR / 'OVMF_VARS.fd'

    if not ovmf_code.is_file():
        raise FileNotFoundError(f'OVMF_CODE.fd not found in the `{OVMF_DIR}` directory')

    qemu_flags = [
        # Disable default devices.
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
        # Create AHCI controller.
        '-device', 'ahci,id=ahci,multifunction=on',
        # Mount a local directory as a FAT partition.
        '-drive', f'if=none,format=raw,file=fat:rw:{ESP_DIR},id=esp',
        '-device', 'ide-drive,bus=ahci.0,drive=esp',
        # Enable the debug connection to allow retrieving test data from the test runner.
        '-debugcon', f'file:{LOG_FILE}', '-global', 'isa-debugcon.iobase=0xE9',
        # Only enable when debugging UEFI boot:
        #'-debugcon', 'file:debug.log', '-global', 'isa-debugcon.iobase=0x402',
    ]

    sp.run([QEMU] + qemu_flags).check_returncode()

def main(args) -> int:
    'Runs the user-requested actions.'

    # Clear any Rust flags which might affect the build.
    os.environ['RUSTFLAGS'] = ''

    # Temporary solution for https://github.com/rust-lang/cargo/issues/4905
    os.environ['RUST_TARGET_PATH'] = str(WORKSPACE_DIR / 'tests')

    print(os.environ['RUST_TARGET_PATH'])

    if len(args) < 2:
        print("Expected at least one parameter (the commands to run): build / doc / run / clippy")
        return 1

    cmds = args[1:]

    KNOWN_CMDS = {
        'build': build,
        'doc': doc,
        'run': run_qemu,
        'clippy': clippy,
    }

    for cmd in cmds:
        if cmd in KNOWN_CMDS:
            try:
                KNOWN_CMDS[cmd]()
            except sp.CalledProcessError:
                return 1
        else:
            print("Unknown verb:", cmd)
            return 1

    return 0

if __name__ == '__main__':
    sys.exit(main(sys.argv))
