# Running the tests

This file documents the process of building and running the test suite.

## Prerequisites

Besides all the requirements for building a UEFI app, you will also need:

- [QEMU](https://www.qemu.org/): the most recent version of QEMU is recommended.
- [Python 3](https://www.python.org): at least version 3.6 is required.
- [OVMF](https://github.com/tianocore/tianocore.github.io/wiki/OVMF):
  You need to extract `OVMF_CODE.fd` and `OVMF_VARS.fd` to the same directory as the `build.py` file.
  Alternatively, install OVMF using your distro's package manager and change the paths in the script file.

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
