# API Guidelines

The `uefi-raw` crate should closely match the definitions in the [UEFI
Specification], with only some light changes to make it more friendly for use in
Rust (e.g. casing follows Rust's conventions and modules are used to provide
some hierarchy). Where the spec is inconsistent or inaccurate, the EDK2 headers
and sources are consulted.

This document describes the API rules in detail. Some of these rules can be
checked with `cargo xtask check-raw`, and that check is run automatically in CI
as well. Other rules require human verification.

If you are contributing to this crate and run into any problems, such as a case
that isn't covered by the rules, or a case where following the rules seems like
it will lead to a bad API, don't hesitate to let us know (e.g. by filing an
[issue]).

## Naming

Type names should match the corresponding spec names, but drop the `EFI_` prefix
and change to `UpperCamelCase` to match Rust's convention. For example,
`EFI_LOAD_FILE_PROTOCOL` becomes `LoadFileProtocol`.

Struct field names and function parameter names should match the corresponding
spec names, but change the case to `snake_case` to match Rust's convention.

When defining a type that isn't part of the spec (for example, a `bitflags!`
type that represents a collection of constants in the spec), prefix the name
with a closely-associated type that is defined in the spec. For example, mode
constants for a `FooBarProtocol` could be collected into a `FooBarMode` type.

It's OK to introduce minor naming changes from the specification where it
improves clarity.

## Layout

All types must be `repr(C)`, `repr(C, packed)`, or `repr(transparent)`.

Types created with the `bitflags!` macro must set `repr(transparent)`.

### Dynamically Sized Types

Some types in the spec end with a variable-length array. It's possible to
represent these as [Dynamically Sized Types], but that should be left to
higher-level APIs. In this crate, add a zero-length array at the end of the
struct to represent the field. For example, if a struct in the spec ends with
`CHAR16 Name[];`, represent that in Rust with `name: [Char16; 0]`.

This pattern of using a `&Header` to work with dynamically-sized data is
rejected by the Stacked Borrows model, but allowed by Tree Borrows. See [UCG
issue 256] for more info.

## Visibility

Everything must have `pub` visibility.

## Constants

Use associated constants where possible instead of top-level constants.

Protocols must have an associated `GUID` constant, for example:

```rust
impl RngProtocol {
    pub const GUID: Guid = guid!("3152bca5-eade-433d-862e-c01cdc291f44");
}
```

## Pointers

Use pointers (`*const`/`*mut`) instead of references (`&`/`&mut`) or wrappers
such as `NonNull`. Raw pointers are nullable, so `OPTIONAL` in the spec does not
change the type of a data pointer.

Function pointers must be `unsafe` and have an explicit ABI (almost always
`efiapi`). If a function pointer field can be null it must be wrapped in
`Option`. Most function pointer fields do not need to allow null pointers
though, unless the spec says otherwise.

### Mutability

Two questions decide the type: who writes through the pointer, and who owns the
pointee. `*mut` means "written by the firmware" or "owned by the caller, who
must free it", whereas `*const` means read-only for both sides. Where that still
leaves a choice, types are modeled for code that calls into UEFI, but they
should remain usable for UEFI _implementations_.

Pointer fields in structs follow the same rule. Pointers to firmware-owned
structures that consumers must only read, such as the `Mode` pointer of a
protocol, are `*const` even where the spec has no `CONST`. UEFI implementations
that update such a structure can use `cast_mut`.

In function parameters, pick between `*const` and `*mut` based on how the
parameter is described in the spec. An `OUT` or `IN OUT` pointer must be
`*mut`. An `IN` pointer is `*const` if the parameter is described as being
source data, but `*mut` if the spec text says that the pointee is written, for
example `Buffer` of `EFI_LOAD_FILE_PROTOCOL.LoadFile()`.

Pointer mutability (`*mut` vs `*const`) is not a UB concern the way reference
mutability is. In general, it is not UB to `cast_mut` a const pointer and write
through it. So picking `*mut` vs `*const` is more about semantics.

### Mapping of C Qualifiers

The spec annotates parameters with `IN`, `OUT`, `IN OUT`, `CONST` and
`OPTIONAL`. In the following tables, `T` is a data type and `F` is a function
pointer type such as `EFI_EVENT_NOTIFY`.

| C                                | Rust                                     |
|----------------------------------|------------------------------------------|
| `IN T Name`                      | `T`                                      |
| `IN T *Name`, `IN CONST T *Name` | `*const T`                               |
| `IN T *Name OPTIONAL`            | `*const T`                               |
| `OUT T *Name`, `IN OUT T *Name`  | `*mut T`                                 |
| `OUT T *Name OPTIONAL`           | `*mut T`                                 |
| `IN F Name`                      | `unsafe extern "efiapi" fn(...)`         |
| `IN F Name OPTIONAL`             | `Option<unsafe extern "efiapi" fn(...)>` |

Buffers (`VOID *Buffer` plus a length) follow the same rule: `*const c_void` if
the firmware reads them, `*mut c_void` if it writes them. Pointers to pointers
depend on who owns each level:

| C                        | Rust                   | Explanation                           |
|--------------------------|------------------------|---------------------------------------|
| `OUT T **Name`           | `*mut *mut T`          | callee allocates, caller frees        |
| `OUT T **Name`           | `*mut *const T`        | points into firmware-owned memory     |
| `OUT T **Name`           | `*const *mut T`        | caller's array, firmware fills it     |
| `IN OUT VOID *Name[N]`   | `*const *mut c_void`   | caller's buffers, firmware fills them |
| `IN CONST CHAR16 **Name` | `*const *const Char16` | read-only strings                     |

This covers arrays too; the count parameter says how many elements the caller
owns, as in `LocateHandleBuffer`. `ProtocolsPerHandle` combines two rows, a
callee-allocated array of pointers into firmware-owned memory, hence
`*mut *mut *const Guid`.

The same rule applies to return values: a pool allocation the caller must free
is returned as `*mut T`, but firmware-owned memory as `*const T`.

The `This` parameter of a protocol function is `*const Self` if the function
only reads state, such as `QueryMode()` of `EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL`.
If the function logically modifies state, such as `Reset()` or `SetMode()`, it
is `*mut Self`, even if the protocol structure itself is not written.

Some inconsistencies will remain. Where these rules do not settle a case, it is
decided case-by-case by what works with real firmware.

## Allowed top-level items

The allowed top-level items are `const`, `impl`, `macro`, `struct`, and
`type`.

Rust `enum`s are not allowed; use the `bitflags!` or `newtype_enum!` macros
instead.

[UEFI Specification]: https://uefi.org/specifications
[issue]: https://github.com/rust-osdev/uefi-rs/issues/new
[Dynamically Sized Types]: https://doc.rust-lang.org/reference/dynamically-sized-types.html
[UCG issue 256]: https://github.com/rust-lang/unsafe-code-guidelines/issues/256
