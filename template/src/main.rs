#![no_main]
#![no_std]

use uefi::prelude::*;

#[entry]
fn main(_handle: Handle, system_table: SystemTable<Boot>) -> Status {
    uefi::helpers::init().unwrap();

    Status::SUCCESS
}
