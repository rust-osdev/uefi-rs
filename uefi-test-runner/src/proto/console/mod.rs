use uefi::table::SystemTable;

pub fn test(st: &SystemTable) {
    info!("Testing console protocols");

    stdout::test(st.stdout());

    let bt = st.boot;
    serial::test(bt);
    gop::test(st);
    pointer::test(bt);
}

mod gop;
mod pointer;
mod serial;
mod stdout;
