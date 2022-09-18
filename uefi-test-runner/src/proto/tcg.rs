use uefi::proto::tcg::{v1, v2};
use uefi::table::boot::BootServices;

// Environmental note:
//
// QEMU does not support attaching multiple TPM devices at once, so we
// can't test TPM v1 and v2 at the same time. To ensure that the CI
// tests both v1 and v2, we arbitrarily choose x86_64 to test TPM v1 and
// x86 32-bit to test TPM v2.
//
// This is enforced in the tests here; if the `ci` feature is enabled
// but the TPM device of the appropriate type isn't available, the test
// panics. If the `ci` feature is not enabled then you can freely enable
// v1, v2, or no TPM.

pub fn test_tcg_v1(bt: &BootServices) {
    info!("Running TCG v1 test");

    let handle = if let Ok(handle) = bt.get_handle_for_protocol::<v1::Tcg>() {
        handle
    } else if cfg!(all(feature = "ci", target_arch = "x86_64")) {
        panic!("TPM v1 is required on x86_64 CI");
    } else {
        info!("No TCG handle found");
        return;
    };

    let mut tcg = bt
        .open_protocol_exclusive::<v1::Tcg>(handle)
        .expect("failed to open TCG protocol");

    let status = tcg.status_check().expect("failed to call status_check");
    info!(
        "tcg status: {:?} {}",
        status.protocol_capability, status.feature_flags
    );
    for event in status.event_log.iter() {
        info!("PCR {}: {:?}", event.pcr_index().0, event.event_type());
    }
}

pub fn test_tcg_v2(bt: &BootServices) {
    info!("Running TCG v2 test");

    let handle = if let Ok(handle) = bt.get_handle_for_protocol::<v2::Tcg>() {
        handle
    } else if cfg!(all(feature = "ci", target_arch = "x86")) {
        panic!("TPM v2 is required on x86 (32-bit) CI");
    } else {
        info!("No TCG handle found");
        return;
    };

    let mut tcg = bt
        .open_protocol_exclusive::<v2::Tcg>(handle)
        .expect("failed to open TCG protocol");

    let capability = tcg.get_capability().expect("failed to call get_capability");
    info!("capability: {:?}", capability);
}

pub fn test(bt: &BootServices) {
    test_tcg_v1(bt);
    test_tcg_v2(bt);
}
