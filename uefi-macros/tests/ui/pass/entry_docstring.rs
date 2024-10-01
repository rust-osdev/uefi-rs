use uefi::{entry, Status};

/// Docstring.
#[entry]
fn efi_main() -> Status {
    Status::SUCCESS
}

// trybuild requires a `main` function.
fn main() {}
