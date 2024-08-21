# Combining Rust `std` with `uefi`

## TL;DR

In Mid-2024, we recommend to stick to our normal guide. Use this document as
guide and outlook for the future of UEFI and Rust.

## About

Programs created with the `uefi` crate are typically created with `#![no_std]`
and `#![no_main]`. A `#![no_std]` crate can use the `core` and `alloc` parts of
Rust's standard library, but not `std`. A `#![no_main]` executable does not use
the standard main entry point, and must define its own entry point; `uefi`
provides the `#[entry]` macro for this purpose.

Rust has added partial support for building UEFI executables without
`#![no_std]` and `#![no_main]`, thus, the standard way. Some functionality
requires a nightly toolchain, they are gated by the `uefi_std` feature (Rust
language feature, not `uefi` crate feature). Follow the
[tracking issue](https://github.com/rust-lang/rust/issues/100499) for details.

## Code Example

Please refer to [`<repo>/uefi-std-example`](/uefi-std-example/README.md) to
see a specific example. The relevant `main.rs` looks as follows:

```rust
{{#include ../../../uefi-std-example/src/main.rs}}
```
