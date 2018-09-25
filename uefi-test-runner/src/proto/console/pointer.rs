use uefi::prelude::*;
use uefi::proto::console::pointer::Pointer;
use uefi::table::boot::BootServices;

use uefi_exts::BootServicesExt;

pub fn test(bt: &BootServices) {
    info!("Running pointer protocol test");
    if let Some(mut pointer) = bt.find_protocol::<Pointer>() {
        let pointer = unsafe { pointer.as_mut() };

        pointer
            .reset(false)
            .expect("Failed to reset pointer device");

        match pointer.state() {
            Ok(state) => info!("Pointer State: {:#?}", state),
            Err(Status::NotReady) => info!("Pointer state has not changed"),
            Err(e) => panic!("Failed to retrieve pointer state ({:?})", e),
        };
    } else {
        warn!("No pointer device found");
    }
}
