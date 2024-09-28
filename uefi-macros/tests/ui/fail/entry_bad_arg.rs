#![no_main]

use uefi::prelude::*;

#[entry]
fn main(_x: usize) -> Status {
    Status::SUCCESS
}
