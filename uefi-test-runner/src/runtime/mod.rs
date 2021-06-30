use uefi::table::runtime::RuntimeServices;

pub fn test(rt: &RuntimeServices) {
    info!("Testing runtime services");
    vars::test(rt);
}

mod vars;
