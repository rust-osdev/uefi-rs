#![no_main]
#![feature(abi_efiapi)]

use uefi::prelude::*;
use uefi_macros::entry;

#[entry]
fn good_entry(_handle: Handle, _st: SystemTable<Boot>) -> Status {
    Status::SUCCESS
}

#[entry(some_arg)]
fn bad_attr_arg(_handle: Handle, _st: SystemTable<Boot>) -> Status {
    Status::SUCCESS
}

#[entry]
extern "C" fn bad_abi_modifier(_handle: Handle, _st: SystemTable<Boot>) -> Status {
    Status::SUCCESS
}

#[entry]
async fn bad_async(_handle: Handle, _st: SystemTable<Boot>) -> Status {
    Status::SUCCESS
}

#[entry]
fn bad_const(_handle: Handle, _st: SystemTable<Boot>) -> Status {
    Status::SUCCESS
}

#[entry]
fn bad_generic<T>(_handle: Handle, _st: SystemTable<Boot>) -> Status {
    Status::SUCCESS
}

#[entry]
fn bad_args(_handle: Handle, _st: SystemTable<Boot>, _x: usize) -> bool {
    false
}

#[entry]
fn bad_return_type(_handle: Handle, _st: SystemTable<Boot>) -> bool {
    false
}
