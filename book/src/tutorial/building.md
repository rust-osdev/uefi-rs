# Building

## Nightly toolchain

Rust's nightly toolchain is currently required because uefi-rs uses some
unstable features.

The easiest way to set this up is using a [rustup toolchain file]. In
the root of your repository, add `rust-toolchain.toml`:

```toml
[toolchain]
channel = "nightly"
targets = ["x86_64-unknown-uefi"]
```

Here we have specified the `x86_64-unknown-uefi` target; there are also
`i686-unknown-uefi` and `aarch64-unknown-uefi` targets available.

Note that nightly releases can sometimes break, so you might opt to pin
to a specific release. For example, `channel = "nightly-2022-11-10"`.

## Build the application

Run this command to build the application:

```sh
cargo build --target x86_64-unknown-uefi
```

This will produce an x86-64 executable:
`target/x86_64-unknown-uefi/debug/my-uefi-app.efi`.

[rustup toolchain file]: https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file
