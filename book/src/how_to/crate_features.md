# Optional Crate Features

There are several optional crate features provided by the `uefi` crate.

- `alloc`: Enables functionality requiring the `alloc` crate from the Rust standard library.
  - For example, this allows many convenient `uefi-rs` functions to operate on heap data (`Box`).
  - It is up to the user to provide a `#[global allocator]`.
- `global_allocator`: implements a `#[global allocator]` using UEFI functions.
  - This allows you to use all abstractions from the `alloc` crate from the Rust standard library
    during runtime. Hence, `Vec`, `Box`, etc. will be able to allocate memory.
    **This is optional**, so you can provide a custom `#[global allocator]` as well.
  - There's no guarantee of the efficiency of UEFI's allocator.
- `logger`: logging implementation for the standard [`log`] crate.
  - Prints output to the UEFI boot services standard text output.
  - No buffering is done: this is not a high-performance logger.
