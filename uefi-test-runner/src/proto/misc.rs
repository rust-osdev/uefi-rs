// SPDX-License-Identifier: MIT OR Apache-2.0

use uefi::prelude::*;
use uefi::proto::misc::ResetNotification;
use uefi::runtime::ResetType;

pub fn test() {
    test_reset_notification();
}

pub fn test_reset_notification() {
    info!("Running loaded ResetNotification protocol test");

    let handle = boot::get_handle_for_protocol::<ResetNotification>()
        .expect("Failed to get handles for `ResetNotification` protocol");

    let mut reset_notif_proto = boot::open_protocol_exclusive::<ResetNotification>(handle)
        .expect("Founded ResetNotification Protocol but open failed");

    // value efi_reset_fn is the type of ResetSystemFn, a function pointer
    unsafe extern "efiapi" fn efi_reset_fn(
        rt: ResetType,
        status: Status,
        data_size: usize,
        data: *const u8,
    ) {
        info!("Inside the event callback, hi, efi_reset_fn");
        info!("rt: {:?} status: {:?}", rt, status);
        info!("size: {:?} data: {:?}", data_size, data);
        // do what you want
    }

    let result = reset_notif_proto.register_reset_notify(efi_reset_fn);
    info!(
        "ResetNotification Protocol register efi_reset_fn test: {:?}",
        result
    );

    let result = reset_notif_proto.unregister_reset_notify(efi_reset_fn);
    info!(
        "ResetNotification Protocol unregister efi_reset_fn test: {:?}",
        result
    );
}
