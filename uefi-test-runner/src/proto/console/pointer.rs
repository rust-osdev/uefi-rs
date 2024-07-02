use uefi::proto::console::pointer::Pointer;
use uefi::table::boot::BootServices;

pub fn test(bt: &BootServices) {
    info!("Running pointer protocol test");
    if let Ok(handle) = bt.get_handle_for_protocol::<Pointer>() {
        let mut pointer = bt
            .open_protocol_exclusive::<Pointer>(handle)
            .expect("failed to open pointer protocol");

        pointer
            .reset(false)
            .expect("Failed to reset pointer device");

        let state = pointer
            .read_state()
            .expect("Failed to retrieve pointer state");

        if let Some(state) = state {
            info!("New pointer State: {:#?}", state);
        } else {
            info!("Pointer state has not changed since the last query");
        }
    } else {
        warn!("No pointer device found");
    }
}
