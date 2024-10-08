# API migration: Deprecating SystemTable/BootServices/RuntimeServices

We are in the process of introducing a significant API change in the `uefi`
crate. We are transitioning away from modeling UEFI tables with structs, and
instead providing an API based on freestanding functions. These functions make
use of a global system table pointer that is set automatically by the `entry`
macro.

A short example:

```rust
// Old API:
use uefi::table::boot::{BootServices, HandleBuffer};
fn find_loaded_image_handles(bt: &BootServices) -> Result<HandleBuffer> {
    bt.locate_handle_buffer(SearchType::from_proto::<LoadedImage>())
}

// New API:
use uefi::boot::{self, HandleBuffer};
fn find_loaded_image_handles() -> Result<HandleBuffer> {
    boot::locate_handle_buffer(SearchType::from_proto::<LoadedImage>())
}
```

The new functions generally have almost the same signature as the methods they
are replacing, so in most cases migration should be as simple as updating
imports and calling the freestanding function instead of a method on
`SystemTable`, `BootServices`, or `RuntimeServices`.

You can retrieve a global `SystemTable` with `uefi::table::system_table_boot` or
`uefi::table::system_table_runtime` to help ease the transition.

If you run into any issues with this migration, please feel free to chat with us
on [Zulip] or file an [issue].

## Timeline

In uefi-0.31, the new API was introduced alongside the old struct-based API.

In uefi-0.32, the old struct-based API was deprecated. In addition, some
breaking changes were made to the API to remove `BootServices` parameters from
various functions.

In uefi-0.33, the deprecated parts off the API were deleted.

## Reason for the change

See [issue #893][RFC] for the discussion that lead to this change.

### Safety of `exit_boot_services`

One of the main motivations for the old API was to make transitioning from boot
services to runtime services a safe operation. Calling `exit_boot_services`
would consume `SystemTable<Boot>` and return a `SystemTable<Runtime>`, ensuring
that it was no longer possible to call boot services methods.

That was the theory, but in practice it didn't always work. Many real-world
programs had to call `SystemTable::unsafe_clone` in order to get another handle
to the system table, and that immediately reintroduced the possibility of
unintentionally accessing boot services after calling `exit_boot_services`.

In addition, there are a great many kinds of resources that should not be
accessed after calling `exit_boot_services`, so even if the `SystemTable<Boot>`
was gone, it's very hard to _statically_ ensure that all references to
boot-services resources are dropped.

Realistically the `exit_boot_services` operation is just too complex to model as
part of Rust's safety guarantees. So in the end, we decided it is better to make
`exit_boot_services` an `unsafe` operation. We do make use of runtime checks
when possible to help catch mistakes (for example, calling a `boot` function
after exiting boot services will panic).

### API complexity

Some parts of the API need to free a pool allocation on drop, or do some other
type of resource cleanup. [`DevicePathToText`] is one example. The user has to
pass in a reference to `BootServices`, and that means the object containing the
allocation needs to hang on to that reference, so it needs a lifetime
parameter. That may "infect" other parts of the API, requiring adding references
and lifetimes to calling functions and to types containing the returned
value. By using a global table pointer instead, this complexity is hidden and
the API becomes simpler.

[`DevicePathToText`]: https://docs.rs/uefi/latest/uefi/proto/device_path/text/struct.DevicePathToText.html
[RFC]: https://github.com/rust-osdev/uefi-rs/issues/893
[Zulip]: https://rust-osdev.zulipchat.com/#narrow/stream/426438-uefi-rs
[issue]: https://github.com/rust-osdev/uefi-rs/issues/new
