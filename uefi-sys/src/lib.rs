#![no_std]
#![feature(abi_efiapi)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod data_types;

use data_types::*;

/*
    We manually generate this type because of a rust issue
    with nested structure containing nested #[align] pragmas
*/
pub type EFI_BOOT_KEY_DATA__bindgen_ty_1 = u32;

include!(concat!(env!("OUT_DIR"), "/uefi_spec.rs"));

include!("guid.rs");
include!("memory_descriptor.rs");
