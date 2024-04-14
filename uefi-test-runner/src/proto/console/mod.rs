use uefi::prelude::*;
use uefi::system;

pub fn test(st: &mut SystemTable<Boot>) {
    info!("Testing console protocols");

    system::with_stdout(stdout::test);

    let bt = st.boot_services();
    unsafe {
        serial::test();
        gop::test(bt);
    }
    pointer::test(bt);
}

mod gop;
mod pointer;
mod serial;
mod stdout;
