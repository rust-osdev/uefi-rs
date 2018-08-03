use uefi::Result;
use uefi::table::boot;

use uefi::proto;
use uefi_exts::BootServicesExt;

pub fn protocol_test(bt: &boot::BootServices) -> Result<()> {
    {
        info!("UEFI Protocol Searching test");

        type SearchedProtocol = proto::console::text::Output;

        if let Ok(handles) = bt.find_handles::<SearchedProtocol>() {
            info!("- Number of handles which implement the SimpleTextOutput protocol: {}", handles.len());
        } else {
            error!("Failed to retrieve the list of handles");
        }
    }

    info!("");

    {
        info!("Debug Support Protocol");

        if let Some(mut debug_support_proto) = bt.find_protocol::<proto::debug::DebugSupport>() {
            let debug_support = unsafe { debug_support_proto.as_mut() };

            info!("- Architecture: {:?}", debug_support.arch());
        } else {
            warn!("UEFI debug protocol is not implemented");
        }
    }

    info!("");

    Ok(())
}
