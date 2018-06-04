use uefi::Result;
use uefi::table::boot;

use uefi::proto;
use uefi_utils;

pub fn protocol_test(_bt: &boot::BootServices) -> Result<()> {
    {
        info!("UEFI Protocol Searching test");

        type SearchedProtocol = proto::console::text::Output;

        if let Ok(handles) = uefi_utils::proto::find_handles::<SearchedProtocol>() {
            info!("- Number of handles which implement the SimpleTextOutput protocol: {}", handles.len());
        } else {
            error!("Failed to retrieve the list of handles");
        }
    }

    info!("");

    {
        info!("Debug Support Protocol");

        if let Some(mut debug_support_proto) = uefi_utils::proto::find_protocol::<proto::debug::DebugSupport>() {
            let debug_support = unsafe { debug_support_proto.as_mut() };

            info!("- Architecture: {:?}", debug_support.arch());
        } else {
            warn!("UEFI debug protocol is not implemented");
        }
    }

    info!("");

    Ok(())
}
