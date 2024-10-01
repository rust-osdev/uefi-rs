#![no_main]

use uefi::prelude::*;

#[entry]
extern "C" fn main() -> Status {
    Status::SUCCESS
}
