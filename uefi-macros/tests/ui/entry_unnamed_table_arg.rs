#![allow(unused_imports)]
#![no_main]
#![feature(abi_efiapi)]

use uefi::prelude::*;
use uefi_macros::entry;

#[entry]
fn unnamed_table_arg(_image: Handle, _: SystemTable<Boot>) -> Status {
    Status::SUCCESS
}
