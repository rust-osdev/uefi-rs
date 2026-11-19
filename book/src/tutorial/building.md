# Building

## Nightly toolchain

Rust's nightly toolchain is currently required because uefi-rs uses some
unstable features. The [`build-std`] feature we use to build the
standard libraries is also unstable.

The easiest way to set this up is using a [rustup toolchain file]. In
the root of your repository, add `rust-toolchain.toml`:

```toml
[toolchain]
channel = "nightly"
components = ["rust-src"]
```

Note that nightly releases can sometimes break, so you might opt to pin
to a specific release. For example, `channel = "nightly-2022-09-01"`.

## Build the application

Run this command to build the application:

```sh
cargo build --target x86_64-unknown-uefi \
    -Zbuild-std=core,compiler_builtins,alloc \
    -Zbuild-std-features=compiler-builtins-mem
```

This will produce an x86-64 executable:
`target/x86_64-unknown-uefi/debug/my-uefi-app.efi`.

## Simplifying the build command

The above build command is verbose and not easy to remember. With a bit
of configuration we can simplify it a lot.

Create a `.cargo` directory in the root of the project:

```sh
mkdir .cargo
```

Create `.cargo/config.toml` with these contents:

```toml
[build]
target = "x86_64-unknown-uefi"

[unstable]
build-std = ["core", "compiler_builtins", "alloc"]
build-std-features = ["compiler-builtins-mem"]
```

Now you can build much more simply:

```sh
cargo build
```

[`build-std`]: https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#build-std
[`rust-toolchain.toml`]: https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file
[rustup toolchain file]: https://rust-lang.github.io/rustup/concepts/toolchains.html
