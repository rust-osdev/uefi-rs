# Device Paths

A device path is a very flexible packed data structure for storing paths
to many kinds of device. Note that these device paths are not the same
thing as file system paths, although they can include file system
paths. Like [handles], device paths can be used to uniquely identify
resources such as consoles, mice, disks, partitions, and more. Unlike
[handles], which are essentially opaque pointers, device paths are
variable-length structures that contain parseable information.

The [`uefi::proto::device_path`] module documentation describes the
details of how device paths are encoded.

Device paths can also be converted to and from human-readable text
representations that look like this:
```text
PciRoot(0x0)/Pci(0x1F,0x2)/Sata(0x0,0xFFFF,0x0)/HD(1,MBR,0xBE1AFDFA,0x3F,0xFBFC1)
```

See [`uefi::proto::device_path::text`] for details.

[handles]: handles.md
[`uefi::proto::device_path`]: https://docs.rs/uefi/latest/uefi/proto/device_path/index.html
[`uefi::proto::device_path::text`]: https://docs.rs/uefi/latest/uefi/proto/device_path/text/index.html
