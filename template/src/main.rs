#![no_main]
#![no_std]
#![feature(abi_efiapi)]

use uefi::prelude::*;
use uefi::ResultExt;

#[entry]
fn main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap_success();

    Status::SUCCESS
}
