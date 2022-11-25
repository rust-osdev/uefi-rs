# Drawing to the Screen

This example shows how to draw to the screen using the [graphics output protocol].
The code will a [Sierpiński triangle] using the "chaos game" method.

![screenshot](https://i.imgur.com/0tpjtV6.png)

The core abstraction used here is a linear buffer:
```rust
{{#include ../../../uefi-test-runner/examples/sierpinski.rs:buffer}}
```

This `Buffer` type stores a `Vec` of [`BltPixel`]s, which are BGRX
32-bit pixels (8 bites each for blue, green, and red, followed by 8
unused bits of padding). We use the `pixel` method to alter a single
pixel at a time. This is often not an efficient method; for more complex
graphics you could use a crate like [`embedded-graphics`].

The `Buffer::blit` method calls the graphics output protocol's `blt`
method to copy the buffer to the screen.

Most of the rest of the code is just implementing the algorithm for
drawing the fractal. Here's the full example:

```rust
{{#include ../../../uefi-test-runner/examples/sierpinski.rs:all}}
```

You can run this example from the [uefi-rs] repository with:
```console
cargo xtask run --example sierpinski
```

[Sierpiński triangle]: https://en.wikipedia.org/wiki/Sierpiński_triangle#Chaos_game
[`BltPixel`]: https://docs.rs/uefi/latest/uefi/proto/console/gop/struct.BltPixel.html
[`embedded-graphics`]: https://crates.io/crates/embedded-graphics
[graphics output protocol]: https://docs.rs/uefi/latest/uefi/proto/console/gop/index.html
[uefi-rs]: https://github.com/rust-osdev/uefi-rs
