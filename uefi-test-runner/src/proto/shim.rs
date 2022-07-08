use uefi::proto::shim::ShimLock;
use uefi::table::boot::BootServices;

pub fn test(bt: &BootServices) {
    info!("Running shim lock protocol test");

    if let Ok(handle) = bt.get_handle_for_protocol::<ShimLock>() {
        let shim_lock = bt
            .open_protocol_exclusive::<ShimLock>(handle)
            .expect("failed to open shim lock protocol");

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
