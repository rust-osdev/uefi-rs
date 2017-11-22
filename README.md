# uefi-rs
This library allows you to write [UEFI][uefi] applications in Rust.

UEFI is the successor to the BIOS. It provides an early boot environment for OS loaders
and other low-level applications.

The objective of this library is to provide **safe** and **performant** wrappers for UEFI
interfaces, and allow developers to write idiomatic Rust code.

[uefi]: https://en.wikipedia.org/wiki/Unified_Extensible_Firmware_Interface

## Documentation
This crate's documentation is fairly minimal, and you are encouraged to refer to 
the [UEFI specification][spec] for detailed information.

You can find some example code in the `tests` directory,
as well as use the `build.py` script to generate the documentation.

This repo also contains a `x86_64-uefi.json` file, which is
a custom Rust target for 64-bit UEFI applications.

[spec]: http://www.uefi.org/specifications

## Running the tests
### Prerequisites
- [QEMU](https://www.qemu.org/): the most recent version of QEMU is recommended.
- [Python 3](https://www.python.org)
- [OVMF](https://github.com/tianocore/tianocore.github.io/wiki/OVMF):
  You need to extract `OVMF_CODE.fd` and `OVMF_VARS.fd` to the same directory as the `build.py` file.
  Alternatively, install OVMF using your distro's package manager and change the paths in the script file.
- [Xargo](https://github.com/japaric/xargo): this is essential if you plan to do any sort of cross-platform / bare-bones Rust programming.

### Steps
It's as simple as running the `build.py` script with the `build` and `run` arguments:

```sh
./build.py build run
```

You can also pass `doc` for generating documentation, or `clippy` to run Clippy.

## License
The code in this repository is licensed under the Mozilla Public License 2.
The full text of the license is available in the `LICENSE` file.
