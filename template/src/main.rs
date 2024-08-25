#![no_main]
#![no_std]

use uefi::prelude::*;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    Status::SUCCESS
}
