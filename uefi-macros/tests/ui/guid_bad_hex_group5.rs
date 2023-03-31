use uefi::Guid;
use uefi_macros::guid;

// Error span should point to the fifth group.
const BadHexGroup5: Guid = guid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaG");

fn main() {}
