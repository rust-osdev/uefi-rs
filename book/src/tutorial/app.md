# Creating a UEFI application

## Install dependencies

Follow the [Rust installation instructions] to set up Rust.

## Create a minimal application

Create an empty application and change to that directory:

```sh
cargo new my-uefi-app
cd my-uefi-app
```

Add a few dependencies:

```sh
cargo add log
cargo add uefi --features logger,panic_handler
```

to your `Cargo.toml`. The resulting `Cargo.toml` should look like that:
```toml
[dependencies]
log = "0.4.21"
uefi = { version = "0.34.0", features = [ "panic_handler", "logger" ] }
```

Replace the contents of `src/main.rs` with this:

```rust
{{#include ../../../uefi-test-runner/examples/hello_world.rs:all}}
```

## Walkthrough

Let's look a quick look at what each part of the program is doing,
starting with the `#![...]` lines at the top:

```rust
{{#include ../../../uefi-test-runner/examples/hello_world.rs:features}}
```

This is some boilerplate that all Rust UEFI applications will
need. `no_main` is needed because the UEFI application entry point is
different from the standard Rust `main` function. `no_std` is needed to
turn off the `std` library; the `core` and `alloc` crates can still be
used.

Next up are some `use` lines. Nothing too exciting here; the
`uefi::prelude` module is intended to be glob-imported, and exports a
number of commonly-used macros, modules, and types.

```rust
{{#include ../../../uefi-test-runner/examples/hello_world.rs:use}}
```

Now we get to the UEFI application `main` function, and here things look
a little different from a standard Rust program.

```rust
{{#include ../../../uefi-test-runner/examples/hello_world.rs:entry}}
```

The `main` function in a UEFI application takes no arguments and returns
a [`Status`], which is essentially a numeric error (or success) code
defined by UEFI. The `main` function must be marked with the `#[entry]`
macro.

The first thing we do inside of `main` is initialize the `helpers`
module, which initializes logging:

```rust
{{#include ../../../uefi-test-runner/examples/hello_world.rs:services}}
```

Next we use the standard `log` crate to output "Hello world!". Then we
call `stall` to make the system pause for 10 seconds. This just ensures
you have enough time to see the output.

```rust
{{#include ../../../uefi-test-runner/examples/hello_world.rs:log}}
```

Finally we return `Status::SUCCESS` indicating that everything completed
successfully:

```rust
{{#include ../../../uefi-test-runner/examples/hello_world.rs:return}}
```

[Rust installation instructions]: https://www.rust-lang.org/tools/install
[`Status`]: https://docs.rs/uefi/latest/uefi/struct.Status.html
[`log`]: https://crates.io/crates/log
[handle]: ../concepts/handles_and_protocols.md
[table]: ../concepts/tables.md
