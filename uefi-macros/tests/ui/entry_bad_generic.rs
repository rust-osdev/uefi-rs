#![allow(unused_imports)]
#![no_main]
#![feature(abi_efiapi)]

use uefi::prelude::*;
use uefi_macros::entry;

#[entry]
fn main<T>(_handle: Handle, _st: SystemTable<Boot>) -> Status {
    Status::SUCCESS
}
