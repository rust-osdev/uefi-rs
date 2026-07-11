# Handles and Protocols

Handles and protocols are at the core of what makes UEFI
extensible. Together they are the mechanism by which UEFI can adapt to a
wide array of hardware and boot conditions, while still providing a
consistent interface to drivers and applications.

### Handles

Handles represent resources. A resource might be a physical device such
as a disk drive or USB device, or something less tangible like a loaded
executable. 

A [Handle] is an opaque pointer, so you can't do anything with it
directly. To operate on a handle you have to open a protocol. 

### Protocols

Protocols are interfaces that provide functions to interact with a
resource. For example, the [BlockIO] protocol provides functions to read
and write to block IO devices.

Protocols are only available during the Boot Services [stage]; you can't
access them during the Runtime stage.

The UEFI Specification defines a very large number of protocols. Because
protocols are inherently very diverse, the best place to learn about
individual protocols is the [UEFI Specification]. There are many
chapters covering various protocols. Not all of these protocols are
wrapped by `uefi-rs` yet (contributions welcome!) but many of the most
commonly useful ones are.

See the [Using Protocols] how-to for details of the `uefi-rs` API for
interacting with protocols.

[UEFI Specification]: https://uefi.org/specifications
[stage]: boot_stages.md
[Handle]: https://docs.rs/uefi/latest/uefi/data_types/struct.Handle.html
[BlockIO]: https://docs.rs/uefi/latest/uefi/proto/media/block/struct.BlockIO.html
[Using Protocols]: ../how_to/protocols.md
