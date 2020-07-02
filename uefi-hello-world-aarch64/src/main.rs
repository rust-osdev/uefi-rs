#![no_std]
#![no_main]
#![feature(asm)]
#![feature(abi_efiapi)]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec;
use core::mem;

use uefi::prelude::*;
use uefi::table::boot::MemoryDescriptor;
use uefi::{Completion, Result};

fn main(_image: Handle, _st: SystemTable<Boot>) -> Result {
    let mut map = BTreeMap::new();

    map.insert("hello", "world");
    map.insert("foo", "bar");

    for (k, v) in map.iter() {
        log::info!("{}: {}", k, v);
    }

    Ok(Completion::from(()))
}

#[entry]
fn efi_main(image: Handle, st: SystemTable<Boot>) -> Status {
    // Initialize utilities (logging, memory allocation...)
    uefi_services::init(&st).expect_success("Failed to initialize utilities");

    // Reset the console before running all the other tests.
    st.clone()
        .stdout()
        .reset(false)
        .expect_success("Failed to reset stdout");

    match main(image, st.clone()) {
        Err(err) => panic!("Received an error: {:#?}", err),
        Ok(_ret) => {
            shutdown(image, st);
        }
    }
}

fn shutdown(image: uefi::Handle, st: SystemTable<Boot>) -> ! {
    use uefi::table::runtime::ResetType;

    // Inform the user, and give the user time to read on real hardware
    log::info!("Program successfully exit, shutting down in 10 seconds...");
    st.boot_services().stall(10_000_000);

    // Exit boot services as a proof that it works :)
    let max_mmap_size =
        st.boot_services().memory_map_size() + 8 * mem::size_of::<MemoryDescriptor>();
    let mut mmap_storage = vec![0; max_mmap_size].into_boxed_slice();
    let (st, _iter) = st
        .exit_boot_services(image, &mut mmap_storage[..])
        .expect_success("Failed to exit boot services");

    // Shut down the system
    let rt = unsafe { st.runtime_services() };
    rt.reset(ResetType::Shutdown, Status::SUCCESS, None)
}
