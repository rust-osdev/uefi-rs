# How to contribute to uefi-rs

Pull requests, issues and suggestions are welcome!

The UEFI spec is huge, so there might be some omissions or some missing features.
You should follow the existing project structure when adding new items.

See the top-level [README](README.md) for details about the repository and
`cargo xtask` commands.

## Development workflow

Run the relevant checks locally before opening a pull request:

```shell
cargo xtask build
cargo xtask test # unit tests
cargo xtask run # integration test using QEMU
```

Upstream CI is the final validation.

## Code and documentation style

Write clean, maintainable code and follow the Rust style enforced by `rustfmt`
and Clippy. Check style with:

```shell
cargo xtask fmt
cargo xtask clippy
cargo xtask doc
```

For all new and changed code, add documentation and comments where they
**provide additional value**:

* **Rustdoc** explains the API to its users.
* **Inline comments** explain the code to the reader, especially *why* it is
  written that way.
* **Commit messages** explain the broader context of a change (for more
  information on commit messages, see below).

Before adding a helper or an external dependency, first look for an appropriate
solution in `core`. Avoid new external dependencies when a standard library
solution is suitable.

### Rustdoc

Start each rustdoc comment with a short, complete summary sentence. The summary
should normally fit on one line and must not exceed two lines at 80 columns.
Put additional explanation in a separate paragraph, if necessary.

Use standard sections for API contracts:

- Use `# Arguments` when parameters need explanation beyond their names and
  types. Describe parameters as ``- `name`: description.``
- Use `# Returns` when the return value is not clear from the summary and type.
- Use `# Errors` for meaningful failure conditions of fallible APIs. Link to
  specific UEFI status values where applicable.
- Use `# Panics` and `# Safety` for their respective contracts.
- Use `# Example` for one example and `# Examples` for multiple examples.

## AI-assisted contributions

If an LLM or other AI meaningfully assisted a contribution, disclose that in the
commit message and/or pull request description. A human contributor must review
and understand the submitted code and remains responsible for it. Submitting a
large AI-generated change without that understanding is not acceptable.

## Commits and pull requests

Write commits that form a logical, **reviewable** path from the initial state to
the final state. Keep the final history compact and concise: squash incidental
fixups and other intermediate states before opening a pull request.

Use this subject format for commits:

```text
component: single line description

<optional body explaining _why_ the change is needed>
```

Add a commit-message body that explains the important motivation when it is not
trivial or **why** a change is needed. In the pull request, briefly repeat your
motivation. It is okay to forward the reviewer to the commit messages, which
are the source of truth.

We highly encourage a line width limit of 72 characters for commit message, but
we do not enforce it.

## UEFI pitfalls

Interfacing with a foreign and unsafe API is a difficult exercise in general, and
UEFI is certainly no exception. This section lists some common pain points that
you should keep in mind while working on UEFI interfaces.

### Enums

Rust and C enums differ in many way. One safety-critical difference is that the
Rust compiler assumes that all variants of Rust enums are known at compile-time.
UEFI, on the other hand, features many C enums which can be freely extended by
implementations or future versions of the spec.

These enums must not be interfaced as Rust enums, as that could lead to undefined
behavior. Instead, integer newtypes with associated constants should be used. The
`newtype_enum` macro is provided by this crate to ease this exercise.

### Pointers

Pointer parameters in UEFI APIs come with many safety conditions. Some of these
are usually expected by unsafe Rust code, while others are more specific to the
low-level environment that UEFI operates in:

- Pointers must reference physical memory (no memory-mapped hardware)
- Pointers must be properly aligned for their target type
- Pointers may only be NULL where UEFI explicitly allows for it
- When an UEFI function fails, nothing can be assumed about the state of data
  behind `*mut` pointers.

## Adding new protocols

You should start by [forking this repository][fork] and cloning it.

UEFI protocols are represented in memory as tables of function pointers,
each of which takes the protocol itself as first parameter.

In `uefi-rs`, protocols are simply `struct`s containing `extern "efiapi" fn`s.
It's imperative to add `#[repr(C)]` to ensure the functions are laid out in memory
in the order the UEFI spec requires.

Each protocol also has a Globally Unique Identifier (in the C API, they're usually
found in a `EFI_*_PROTOCOL_GUID` define). In Rust, we store the GUID as an associated
constant, by implementing the unsafe trait `uefi::proto::Identify`. For convenience,
this is done through the `unsafe_protocol` macro.

Finally, you should derive the `Protocol` trait. This is a marker trait,
extending `Identify`, which is used as a generic bound in the functions which retrieve
protocol implementations.

An example protocol declaration:

```rust
/// Protocol which does something.
#[repr(C)]
#[unsafe_protocol("abcdefgh-1234-5678-9012-123456789abc")]
pub struct NewProtocol {
  some_entry_point: extern "efiapi" fn(
    this: *const NewProtocol,
    some_parameter: SomeType,
    some_other_parameter: SomeOtherType,
  ) -> Status,
  some_other_entry_point: extern "efiapi" fn(
    this: *mut NewProtocol,
    another_parameter: AnotherType,
  ) -> SomeOtherResult,
  // ...
}
```

There should also be an `impl` block providing safe access to the functions:

```rust
impl NewProtocol {
  /// This function does something.
  pub fn do_something(&self, a: SomeType, b: SomeOtherType) -> Result {
    // Call the wrapped function
    let status = unsafe { (self.some_entry_point)(self, a, b) };
    // `Status` provides a helper function for converting to `Result`
    status.into()
  }
}
```

[fork]: https://docs.github.com/en/free-pro-team@latest/github/getting-started-with-github/fork-a-repo

## Publishing new versions of the crates

Maintainers of this repository might also be interested in [the guidelines](PUBLISHING.md)
for publishing new versions of the uefi-rs crates.
