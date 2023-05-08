#![allow(unused_imports)]
#![no_main]

use uefi::prelude::*;
use uefi_macros::entry;

#[entry]
extern "C" fn main(_handle: Handle, _st: SystemTable<Boot>) -> Status {
    Status::SUCCESS
}
