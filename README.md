# uefi-rs
This library allows you to write [UEFI][uefi] applications in Rust.

[uefi]: https://en.wikipedia.org/wiki/Unified_Extensible_Firmware_Interface

## Documentation
The best way to learn how to write UEFI applications is to read the [UEFI spec][spec]
and look at the example code in the `tests` directory.

There is also the `x86_64-uefi.json` file, which is
a custom Rust target for 64-bit UEFI applications.

## Running the tests
### Prerequisites
- [QEMU](https://www.qemu.org/)
- [Python 3](https://www.python.org)
- [OVMF](https://github.com/tianocore/tianocore.github.io/wiki/OVMF):
  You need to extract `OVMF_CODE.fd` and `OVMF_VARS.fd` to the same directory as the `build.py` file.
  Alternatively, install OVMF using your distro's package manager and change the paths in the script file.

### Steps
It's as simple as running the `build.py` script with the `build` and `run` arguments:

```sh
./build.py build run
```

You can also pass `doc` for generating documentation, or `clippy` to run Clippy.
