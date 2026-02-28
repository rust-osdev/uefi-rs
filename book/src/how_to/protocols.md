# Using Protocols

The open a protocol, you must first get a handle, then open a protocol
on that handle. See [Handles and Protocols] for an overview of what
these terms mean.

To get a handle you can use:
* [`BootServices::locate_handle_buffer`]: this can be used to get _all_
  available handles, or just the handles that support a particular
  protocol.
* [`BootServices::locate_handle`]: the same as `locate_handle_buffer`,
  but you provide the slice that stores the handles.
* [`BootServices::locate_device_path`]: find a handle by [Device Path].

Once you have obtained a handle, use
[`BootServices::open_protocol_exclusive`] to open a protocol on that
handle. This returns a [`ScopedProtocol`], which automatically closes
the protocol when dropped.

Using [`BootServices::open_protocol_exclusive`] is the safest way to
open a protocol, but in some cases a protocol cannot be opened in
exclusive mode. The `unsafe` [`BootServices::open_protocol`] can be used
in that case.

## Example

For this example we'll look at a program that opens a couple different
protocols. This program opens the [`LoadedImage`] protocol to get
information about an executable (the currently-running program in this
case). It also opens the [`DevicePathToText`] protocol to get the file
system path that the program was launched from.

We'll walk through the details of this program shortly, but first here's
the whole thing:

```rust
{{#include ../../../uefi-test-runner/examples/loaded_image.rs:all}}
```

When the program is run it will print something like this:

```text
[ INFO]: example.rs@058: Image path: \EFI\BOOT\BOOTX64.EFI
```

## Walkthrough

The `main` function looks much like the ["Hello world!" example]. It
sets up logging, calls `print_image_path`, and pauses for ten seconds to
give you time to read the output. Let's look at `print_image_path`:

```rust
{{#include ../../../uefi-test-runner/examples/loaded_image.rs:print_image_path}}
```

The return type is a [`uefi::Result`], which is a `Result` alias that
combines [`uefi::Status`] with the error data. Both the success and
error data types are `()` by default.

The function starts by opening the [`LoadedImage`] protocol:

```rust
{{#include ../../../uefi-test-runner/examples/loaded_image.rs:loaded_image}}
```

The [`open_protocol_exclusive`] method takes a type parameter, which is
the type of [`Protocol`] you want to open ([`LoadedImage`] in this
case). It also takes one regular argument of type [`Handle`]. For this
example we want the handle of the currently-running image, which was
passed in as the first argument to `main`. The handle is conveniently
accessible through [`BootServices::image_handle`], so we use that here.

Next the program opens the [`DevicePathToText`] protocol:

```rust
{{#include ../../../uefi-test-runner/examples/loaded_image.rs:device_path}}
```

This protocol isn't available for the `image_handle`, so we start by
using [`locate_handle_buffer`] to find all handles that support
`DevicePathToText`. We only need one handle though, so we call `first()`
and discard the rest. Then we call [`open_protocol_exclusive`] again. It
looks more or less like the previous time, but with [`DevicePathToText`]
as the type parameter and `device_path_to_text_handle` as the handle.

Now that we have both protocols open, we can use them together to get
the program's path and convert it to text:

```rust
{{#include ../../../uefi-test-runner/examples/loaded_image.rs:text}}
```

Since protocols do a wide range of different things, the methods
available to call are very specific to each individual protocol. The
best places to find out what each protocol can do are the [uefi-rs
reference documentation] and the [UEFI Specification].

[Device Path]: ../concepts/device_paths.md
[Handles and Protocols]: ../concepts/handles_and_protocols.md
[UEFI Specification]: https://uefi.org/specifications
[`BootServices::image_handle`]: https://docs.rs/uefi/latest/uefi/table/boot/struct.BootServices.html#method.image_handle
[`BootServices::locate_device_path`]: https://docs.rs/uefi/latest/uefi/table/boot/struct.BootServices.html#method.locate_device_path
[`BootServices::locate_handle_buffer`]: https://docs.rs/uefi/latest/uefi/table/boot/struct.BootServices.html#method.locate_handle_buffer
[`BootServices::locate_handle`]: https://docs.rs/uefi/latest/uefi/table/boot/struct.BootServices.html#method.locate_handle
[`BootServices::open_protocol`]: https://docs.rs/uefi/latest/uefi/table/boot/struct.BootServices.html#method.open_protocol
[`BootServices::open_protocol_exclusive`]: https://docs.rs/uefi/latest/uefi/table/boot/struct.BootServices.html#method.open_protocol_exclusive
[`BootServices`]: https://docs.rs/uefi/latest/uefi/table/boot/struct.BootServices.html
[`DevicePathToText`]: https://docs.rs/uefi/latest/uefi/proto/device_path/text/struct.DevicePathToText.html
["Hello world!" example]: ../tutorial/app.html
[`Handle`]: https://docs.rs/uefi/latest/uefi/data_types/struct.Handle.html
[`LoadedImage`]: https://docs.rs/uefi/latest/uefi/proto/loaded_image/struct.LoadedImage.html
[`OpenProtocolAttributes::Exclusive`]: https://docs.rs/uefi/latest/uefi/table/boot/enum.OpenProtocolAttributes.html#variant.Exclusive
[`OpenProtocolAttributes`]: https://docs.rs/uefi/latest/uefi/table/boot/enum.OpenProtocolAttributes.html
[`OpenProtocolParams`]: https://docs.rs/uefi/latest/uefi/table/boot/struct.OpenProtocolParams.html
[`Protocol`]: https://docs.rs/uefi/latest/uefi/proto/trait.Protocol.html
[`ScopedProtocol`]: https://docs.rs/uefi/latest/uefi/table/boot/struct.ScopedProtocol.html
[`locate_handle_buffer`]: https://docs.rs/uefi/latest/uefi/table/boot/struct.BootServices.html#method.locate_handle_buffer
[`open_protocol`]: https://docs.rs/uefi/latest/uefi/table/boot/struct.BootServices.html#method.open_protocol
[`open_protocol_exclusive`]: https://docs.rs/uefi/latest/uefi/table/boot/struct.BootServices.html#method.open_protocol_exclusive
[uefi-rs reference documentation]: https://docs.rs/uefi/latest/uefi/proto/index.html
[`uefi::Result`]: https://docs.rs/uefi/latest/uefi/type.Result.html
[`uefi::Status`]: https://docs.rs/uefi/latest/uefi/struct.Status.html
