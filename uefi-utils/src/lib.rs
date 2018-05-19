//! Utility functions for the most common UEFI patterns.

#![feature(alloc)]

#![no_std]

extern crate uefi;
extern crate uefi_services;

#[macro_use]
extern crate alloc;

pub mod proto;

use uefi::table::boot;

fn boot_services() -> &'static boot::BootServices {
    uefi_services::system_table().boot
}
