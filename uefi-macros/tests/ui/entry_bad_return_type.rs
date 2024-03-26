#![allow(unused_imports)]
#![no_main]
#![feature(abi_efiapi)]

use uefi::prelude::*;
use uefi_macros::entry;

#[entry]
fn main(_handle: Handle, _st: SystemTable<Boot>) -> bool {
    false
}
