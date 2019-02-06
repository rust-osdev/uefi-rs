use uefi::prelude::*;
use uefi::proto::console::pointer::Pointer;
use uefi::table::boot::BootServices;

pub fn test(bt: &BootServices) {
    info!("Running pointer protocol test");
    if let Ok(pointer) = bt.locate_protocol::<Pointer>() {
        let pointer = pointer.expect("Warnings encountered while opening pointer protocol");
        let pointer = unsafe { &mut *pointer.get() };

        pointer
            .reset(false)
            .expect_success("Failed to reset pointer device");

        let state = pointer
            .read_state()
            .expect_success("Failed to retrieve pointer state");

        if let Some(state) = state {
            info!("New pointer State: {:#?}", state);
        } else {
            info!("Pointer state has not changed since the last query");
        }
    } else {
        warn!("No pointer device found");
    }
}
