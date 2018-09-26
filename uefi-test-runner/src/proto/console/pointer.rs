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
