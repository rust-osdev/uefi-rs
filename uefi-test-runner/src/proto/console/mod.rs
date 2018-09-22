use uefi::table::SystemTable;

pub fn test(st: &SystemTable) {
    stdout::test(st.stdout());

    let bt = st.boot;
    serial::test(bt);
    gop::test(bt);
    pointer::test(bt);
}

mod stdout;
mod serial;
mod gop;
mod pointer;
