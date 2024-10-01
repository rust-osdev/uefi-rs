use uefi::{entry, Status};

#[entry]
unsafe fn efi_main() -> Status {
    Status::SUCCESS
}

// trybuild requires a `main` function.
fn main() {}
