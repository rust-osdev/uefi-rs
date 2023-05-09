use uefi::table::{Boot, SystemTable};
use uefi::{entry, Handle, Status};

#[entry]
fn efi_main(image: Handle, st: SystemTable<Boot>) -> Status {
    Status::SUCCESS
}

// trybuild requires a `main` function.
fn main() {}
