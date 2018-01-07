#!/usr/bin/env python3

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

# Path to directory containing `OVMF_{CODE/VARS}.fd`.
# TODO: use installed OVMF, if available.
OVMF_DIR = Path('.')

# Path to workspace's `Cargo.toml`
WORKSPACE_DIR = Path(__file__).resolve().parents[1]
BUILD_DIR = WORKSPACE_DIR / Path('target') / TARGET / CONFIG
ESP_DIR = BUILD_DIR / 'esp'

def run_xargo(verb, *flags):
    sp.run([XARGO, verb, '--target', TARGET] + list(flags)).check_returncode()

def build():
    run_xargo('build', '--package', 'tests')

    input_lib = BUILD_DIR / 'libtests.a'

    boot_dir = ESP_DIR / 'EFI' / 'Boot'
    boot_dir.mkdir(parents=True, exist_ok=True)

    output = boot_dir / 'BootX64.efi'

    sp.run([LINKER] + LINKER_FLAGS + [str(input_lib), '-Out:{}'.format(output)]).check_returncode()

def doc():
    run_xargo('doc', '--no-deps', '--package', 'uefi')

def clippy():
    run_xargo('clippy')


def run_qemu():
    ovmf_code, ovmf_vars = OVMF_DIR / 'OVMF_CODE.fd', OVMF_DIR / 'OVMF_VARS.fd'

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
        '-drive', 'if=pflash,format=raw,file={},readonly=on'.format(ovmf_code),
        '-drive', 'if=pflash,format=raw,file={},readonly=on'.format(ovmf_vars),
        # Create AHCI controller.
        '-device', 'ahci,id=ahci,multifunction=on',
        # Mount a local directory as a FAT partition.
        '-drive', 'if=none,format=raw,file=fat:rw:{},id=esp'.format(ESP_DIR),
        '-device', 'ide-drive,bus=ahci.0,drive=esp',
        # Only enable when debugging UEFI boot:
        #'-debugcon', 'file:debug.log', '-global', 'isa-debugcon.iobase=0x402',
    ]

    sp.run([QEMU] + qemu_flags).check_returncode()


def main(args) -> int:
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
    }

    for cmd in cmds:
        if cmd in KNOWN_CMDS:
            KNOWN_CMDS[cmd]()
        else:
            print("Unknown verb:", cmd)
            return 1

if __name__ == '__main__':
    sys.exit(main(sys.argv))
