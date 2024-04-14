#![no_main]
#![no_std]

use uefi::prelude::*;

#[entry]
fn main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi::helpers::init(&mut system_table).unwrap();

    Status::SUCCESS
}
