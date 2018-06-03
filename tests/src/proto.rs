use uefi::Result;
use uefi::table::boot;

use uefi::proto;
use uefi_utils;

pub fn protocol_test(_bt: &boot::BootServices) -> Result<()> {
    type SearchedProtocol = proto::console::text::Output;

    let handles = uefi_utils::proto::find_handles::<SearchedProtocol>()
        .expect("Failed to retrieve the list of handles");

    info!("Number of handles which implement the SimpleTextOutput protocol: {}", handles.len());

    let mut debug_support_proto = uefi_utils::proto::find_protocol::<proto::debug::DebugSupport>()
        .expect("UEFI debug protocol is not implemented");

    let debug_support = unsafe { debug_support_proto.as_mut() };

    info!("{:#?}", debug_support.arch());

    Ok(())
}
