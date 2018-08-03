use uefi::table::SystemTable;

pub fn test(st: &SystemTable) {
    stdout::test(st.stdout());

    let bt = st.boot;
    gop::test(bt);
    pointer::test(bt);
}

mod stdout;
mod gop;
mod pointer;
