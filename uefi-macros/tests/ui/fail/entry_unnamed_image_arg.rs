#![allow(unused_imports)]
#![no_main]

use uefi::prelude::*;
use uefi_macros::entry;

#[entry]
fn unnamed_image_arg(_: Handle, _st: SystemTable<Boot>) -> Status {
    Status::SUCCESS
}
