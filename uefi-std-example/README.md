# Minimal Rust App using `std` and `uefi`

Minimal example of a "standard Rust application" that showcases how `uefi` can
be utilized and enhance the developers experience, when `std` is available.

For simplicity, this example is minimal and the documentation is focused on
`x86_64-unknown-uefi`. However, it works similar for other supported UEFI
platforms.

## Build

Build the app using
`$ cargo +nightly build --target x86_64-unknown-uefi`. To build it from the root
directory (the Cargo workspace), append `-p uefi-std-example`.

## Run

The resulting `.efi` file can be found in `target/x86_64-unknown-uefi/<debug|release>/uefi-std-example.efi`.
