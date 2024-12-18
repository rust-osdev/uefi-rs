# Building drivers

There are [three types][spec-images] of UEFI images:
* Application
* Boot service driver
* Runtime driver

[By default][target-flag], Rust's UEFI targets produce applications. This can be
changed by passing a [`subsystem`] linker flag in `rustflags` and setting the
value to `efi_boot_service_driver` or `efi_runtime_driver`.

Example:

```rust
// In build.rs

fn main() {
    let target = std::env::var("TARGET").unwrap();
    if target.ends_with("-unknown-uefi") {
        println!("cargo::rustc-link-arg=/subsystem:efi_runtime_driver");
    }
}
```

[spec-images]: https://uefi.org/specs/UEFI/2.10/02_Overview.html#uefi-images
[target-flag]: https://github.com/rust-lang/rust/blob/f4d794ea0b845413344621d89f6c945062748485/compiler/rustc_target/src/spec/base/uefi_msvc.rs#L33
[`subsystem`]: https://learn.microsoft.com/en-us/cpp/build/reference/subsystem?view=msvc-170
