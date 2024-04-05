use uefi::prelude::*;
use uefi::proto::misc::{ResetNotification, Timestamp};
use uefi::table::runtime;

///
/// you may see those log, it's nothing just for your computer firmware does not support the new UEFI feature.
///
/// ```sh
/// [ INFO]: uefi-test-runner\src\proto\misc.rs@012: Running loaded Timestamp Protocol test
/// [ WARN]: uefi-test-runner\src\proto\misc.rs@026: Failed to open Timestamp Protocol: Error { status: UNSUPPORTED, data: () }
/// [ INFO]: uefi-test-runner\src\proto\misc.rs@033: Running loaded ResetNotification protocol test
/// [ WARN]: uefi-test-runner\src\proto\misc.rs@068: Failed to open ResetNotification Protocol: Error { status: UNSUPPORTED, data: () }
/// ```
pub fn test(image: Handle, bt: &BootServices) {
    test_timestamp(image, bt);
    test_reset_notification(image, bt);
}

pub fn test_timestamp(image: Handle, bt: &BootServices) {
    info!("Running loaded Timestamp Protocol test");

    let result = bt
        .open_protocol_exclusive::<Timestamp>(image);

    match result {
        Ok(timestamp_proto) => {
            let timestamp = timestamp_proto.get_timestamp();
            info!("Timestamp Protocol's timestamp: {:?}", timestamp);

            let properties = timestamp_proto.get_properties();
            info!("Timestamp Protocol's properties: {:?}", properties);
        }
        Err(err) => {
            warn!("Failed to open Timestamp Protocol: {:?}", err);
        }
    }
}


pub fn test_reset_notification(image: Handle, bt: &BootServices) {
    info!("Running loaded ResetNotification protocol test");

    let result = bt
        .open_protocol_exclusive::<ResetNotification>(image);

    match result {
        Ok(mut reset_notif_proto) => {
            let result = reset_notif_proto.register_reset_notify(None);
            info!("ResetNotification Protocol register null test: {:?}", result);

            let result = reset_notif_proto.unregister_reset_notify(None);
            info!("ResetNotification Protocol unregister null test: {:?}", result);



            // value efi_reset_fn is the type of ResetSystemFn, a function pointer
            unsafe extern "efiapi" fn efi_reset_fn(
                rt: runtime::ResetType,
                status: Status,
                data_size: usize,
                data: *const u8,
            ) {
                info!("Inside the event callback, hi, efi_reset_fn");
                info!("rt: {:?} status: {:?}", rt, status);
                info!("size: {:?} data: {:?}", data_size, data);
                // do what you want
            }

            let result = reset_notif_proto.register_reset_notify(Some(efi_reset_fn));
            info!("ResetNotification Protocol register efi_reset_fn test: {:?}", result);

            let result = reset_notif_proto.unregister_reset_notify(Some(efi_reset_fn));
            info!("ResetNotification Protocol unregister efi_reset_fn test: {:?}", result);
        }
        Err(err) => {
            warn!("Failed to open ResetNotification Protocol: {:?}", err);
        }
    }
}

