use uefi_macros::unsafe_guid;

// The GUID here is OK.
#[unsafe_guid("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")]
struct Good;

// Fail because the length is wrong.
#[unsafe_guid("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaa")]
struct TooShort;

// Error span should point to the second group.
#[unsafe_guid("aaaaaaaa-Gaaa-aaaa-aaaa-aaaaaaaaaaaa")]
struct BadHexGroup2;

// Error span should point to the fifth group.
#[unsafe_guid("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaG")]
struct BadHexGroup5;

fn main() {}
