use uefi::table::boot::BootServices;
use uefi::proto::console::pointer::Pointer;

use uefi_exts::BootServicesExt;

pub fn test(bt: &BootServices) {
    if let Some(mut pointer) = bt.find_protocol::<Pointer>() {
        let pointer = unsafe { pointer.as_mut() };

        pointer.reset(false).expect("Failed to reset pointer device");

        if let Ok(state) = pointer.state() {
            info!("Pointer State: {:#?}", state);
        } else {
            error!("Failed to retrieve pointer state");
        }
    } else {
        warn!("No pointer device found");
    }
}
