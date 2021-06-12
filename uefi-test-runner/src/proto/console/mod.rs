use uefi::prelude::*;

pub fn test(st: &mut SystemTable<Boot>) {
    info!("Testing console protocols");

    stdout::test(st.stdout());

    let bt = st.boot_services();
    serial::test(bt);
    gop::test(bt);
    pointer::test(bt);
}

mod gop;
mod pointer;
mod serial;
mod stdout;
