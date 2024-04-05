use uefi::prelude::*;
use uefi::proto::misc::{ResetNotification, Timestamp};
use uefi::table::runtime;

///
/// you may see those log, it's nothing just for your computer firmware does not support the new UEFI feature of Timestamp Protocol.
///
/// ```sh
/// [ INFO]: uefi-test-runner\src\proto\misc.rs@020: Running loaded Timestamp Protocol test
/// [ WARN]: uefi-test-runner\src\proto\misc.rs@037: Failed to found Timestamp Protocol: Error { status: NOT_FOUND, data: () }
/// [ INFO]: uefi-test-runner\src\proto\misc.rs@043: Running loaded ResetNotification protocol test
/// [ INFO]: uefi-test-runner\src\proto\misc.rs@053: ResetNotification Protocol register null test: Err(Error { status: INVALID_PARAMETER, data: () })
/// [ INFO]: uefi-test-runner\src\proto\misc.rs@059: ResetNotification Protocol unregister null test: Err(Error { status: INVALID_PARAMETER, data: () })
/// [ INFO]: uefi-test-runner\src\proto\misc.rs@078: ResetNotification Protocol register efi_reset_fn test: Ok(())
/// [ INFO]: uefi-test-runner\src\proto\misc.rs@084: ResetNotification Protocol unregister efi_reset_fn test: Ok(())
/// ```
pub fn test(bt: &BootServices) {
    test_timestamp(bt);
    test_reset_notification(bt);
}

pub fn test_timestamp(bt: &BootServices) {
    info!("Running loaded Timestamp Protocol test");

    let handle = bt.get_handle_for_protocol::<Timestamp>();

    match handle {
        Ok(handle) => {
            let timestamp_proto = bt
                .open_protocol_exclusive::<Timestamp>(handle)
                .expect("Founded Timestamp Protocol but open failed");

            let timestamp = timestamp_proto.get_timestamp();
            info!("Timestamp Protocol's timestamp: {:?}", timestamp);

            let properties = timestamp_proto.get_properties();
            info!("Timestamp Protocol's properties: {:?}", properties);
        }
        Err(err) => {
            warn!("Failed to found Timestamp Protocol: {:?}", err);
        }
    }
}

pub fn test_reset_notification(bt: &BootServices) {
    info!("Running loaded ResetNotification protocol test");

    let handle = bt.get_handle_for_protocol::<ResetNotification>();

    match handle {
        Ok(handle) => {
            let mut reset_notif_proto = bt
                .open_protocol_exclusive::<ResetNotification>(handle)
                .expect("Founded ResetNotification Protocol but open failed");
            let result = reset_notif_proto.register_reset_notify(None);
            info!(
                "ResetNotification Protocol register null test: {:?}",
                result
            );

            let result = reset_notif_proto.unregister_reset_notify(None);
            info!(
                "ResetNotification Protocol unregister null test: {:?}",
                result
            );

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
            info!(
                "ResetNotification Protocol register efi_reset_fn test: {:?}",
                result
            );

            let result = reset_notif_proto.unregister_reset_notify(Some(efi_reset_fn));
            info!(
                "ResetNotification Protocol unregister efi_reset_fn test: {:?}",
                result
            );
        }
        Err(err) => {
            warn!("Failed to found ResetNotification Protocol: {:?}", err);
        }
    }
}
