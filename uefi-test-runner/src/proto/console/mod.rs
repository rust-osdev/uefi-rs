use uefi::prelude::*;
use uefi::system;

pub fn test(image: Handle, st: &mut SystemTable<Boot>) {
    info!("Testing console protocols");

    system::with_stdout(stdout::test);

    let bt = st.boot_services();
    unsafe {
        serial::test();
        gop::test(image, bt);
    }
    pointer::test(bt);
}

mod gop;
mod pointer;
mod serial;
mod stdout;
