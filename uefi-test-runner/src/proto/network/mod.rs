pub fn test() {
    info!("Testing Network protocols");

    pxe::test();
    snp::test();
}

mod pxe;
mod snp;
