# uefi-test-runner

This package is a UEFI application for running tests. It is intended to
be run in a specially-configured QEMU VM. This allows us to test the
parts of the `uefi` package that depend on a UEFI environment, such as
various boot services and protocols.

## Requirements

- [QEMU](https://www.qemu.org/): the most recent version of QEMU is recommended.
- [Python 3](https://www.python.org): at least version 3.6 is required.
- [OVMF](https://github.com/tianocore/tianocore.github.io/wiki/OVMF):
  You need to extract the firmware files into the `uefi-test-runner` directory.
  - For x86_64: `OVMF_CODE.fd` and `OVMF_VARS.fd`
  - For AArch64: `QEMU_EFI-pflash.raw` and `vars-template-pflash.raw`
  Alternatively, install OVMF using your distro's package manager and change the paths in the script file.
  **Note**: if your distro's OVMF version is too old / does not provide these files,
  you can download [Gerd Hoffmann's builds](https://www.kraxel.org/repos/) and extract them in the local directory.

## Build and run in QEMU

Use `cargo xtask run` to build `uefi-test-runner` and run it in QEMU. See
the top-level [README](../README.md) for more details of `cargo xtask`.
