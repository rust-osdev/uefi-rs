# Running the tests

This file documents the process of building and running the test suite.

## Prerequisites

Besides all the [core library requirements](https://github.com/rust-osdev/uefi-rs/blob/master/BUILDING.md#Prerequisites) for building a UEFI app, the tests have additional requirements:

- [QEMU](https://www.qemu.org/): the most recent version of QEMU is recommended.
- [Python 3](https://www.python.org): at least version 3.6 is required.
- [OVMF](https://github.com/tianocore/tianocore.github.io/wiki/OVMF):
  You need to extract the firmware files to the same directory as the `build.py` file.
  - For x86_64: `OVMF_CODE.fd` and `OVMF_VARS.fd`
  - For AArch64: `QEMU_EFI-pflash.raw` and `vars-template-pflash.raw`
  Alternatively, install OVMF using your distro's package manager and change the paths in the script file.
  **Note**: if your distro's OVMF version is too old / does not provide these files,
  you can download [Gerd Hoffmann's builds](https://www.kraxel.org/repos/) and extract them in the local directory.

## Steps

It's as simple as running the `build.py` script with the ``run` argument:

```sh
./build.py run
```

Available commands:

- `build`: only build
- `run`: (re)build and run
- `doc`: generate documentation
- `clippy`: run Clippy

Available options:

- `--target {x86_64,aarch64}`: choose which architecture to build/run the tests
- `--verbose`: enables verbose mode, prints commands before running them
- `--headless`: enables headless mode, which runs QEMU without a GUI
- `--release`: builds the code with optimizations enabled
