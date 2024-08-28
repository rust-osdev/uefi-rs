#![allow(unused_imports)]
#![no_main]
#![allow(deprecated)]

use uefi::prelude::*;
use uefi_macros::entry;

#[entry]
fn main(_handle: Handle, _st: SystemTable<Boot>, _x: usize) -> Status {
    Status::SUCCESS
}
