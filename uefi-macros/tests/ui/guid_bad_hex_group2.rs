use uefi::Guid;
use uefi_macros::guid;

// Error span should point to the second group.
const BadHexGroup2: Guid = guid!("aaaaaaaa-Gaaa-aaaa-aaaa-aaaaaaaaaaaa");

fn main() {}
