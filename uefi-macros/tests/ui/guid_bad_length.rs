use uefi::Guid;
use uefi_macros::guid;

// Fail because the length is wrong.
const TooShort: Guid = guid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaa");

fn main() {}
