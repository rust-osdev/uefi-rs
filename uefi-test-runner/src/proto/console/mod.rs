use uefi::table::BootSystemTable;

pub fn test(st: &BootSystemTable) {
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
