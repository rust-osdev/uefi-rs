# Building

## Toolchain

In order to compile for UEFI, an appropriate target must be installed. The
easiest way to set this up is using a [rustup toolchain file]. In the root of
your repository, add `rust-toolchain.toml`:

```toml
[toolchain]
targets = ["aarch64-unknown-uefi", "i686-unknown-uefi", "x86_64-unknown-uefi"]
```

Here we have specified all three of the currently-supported UEFI targets; you
can remove some if you don't need them.

## Build the application

Run this command to build the application:

```sh
cargo build --target x86_64-unknown-uefi
```

This will produce an x86-64 executable:
`target/x86_64-unknown-uefi/debug/my-uefi-app.efi`.

[rustup toolchain file]: https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file
