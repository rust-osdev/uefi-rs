# GUID

GUID is short for Globally Unique Identifier. A GUID is always 16 bytes,
and has a standard string representation format that looks like this:
`313b0d7c-fed4-4de7-99ed-2fe48874a410`. The details of the GUID format
aren't too important, but be aware that the actual byte representation
is not in the same order as the string representation because the first
three fields are little-endian. For the most part you can treat GUIDs as
opaque identifiers.

The UEFI specification uses GUIDs all over the place. GUIDs are used to
identify protocols, disk partitions, variable groupings, and much
more. In `uefi-rs`, GUIDs are represented by the [`Guid`] type.

[`Guid`]: https://docs.rs/uefi/latest/uefi/struct.Guid.html
