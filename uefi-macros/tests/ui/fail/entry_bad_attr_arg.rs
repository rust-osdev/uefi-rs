#![no_main]

use uefi::prelude::*;

#[entry(some_arg)]
fn main() -> Status {
    Status::SUCCESS
}
