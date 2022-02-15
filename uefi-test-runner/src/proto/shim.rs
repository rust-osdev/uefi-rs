use uefi::proto::shim::ShimLock;
use uefi::table::boot::BootServices;

pub fn test(bt: &BootServices) {
    info!("Running shim lock protocol test");

    if let Ok(shim_lock) = bt.locate_protocol::<ShimLock>() {
        let shim_lock = unsafe { &*shim_lock.get() };

        // An empty buffer should definitely be invalid, so expect
        // shim to reject it.
        let buffer = [];
        shim_lock
            .verify(&buffer)
            .expect_err("shim failed to reject an invalid application");
    } else {
        info!("Shim lock protocol is not supported");
    }
}
