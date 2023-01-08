use alloc::vec::Vec;
use core::mem::MaybeUninit;
use uefi::proto::tcg::{v1, v2, EventType, HashAlgorithm, PcrIndex};
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

/// Get the SHA-1 digest stored in the given PCR index.
fn tcg_v1_read_pcr(tcg: &mut v1::Tcg, pcr_index: PcrIndex) -> v1::Sha1Digest {
    // Input command block.
    #[rustfmt::skip]
    let input = [
        // tag: TPM_TAG_RQU_COMMAND
        0x00, 0xc1,
        // paramSize
        0x00, 0x00, 0x00, 0x0e,
        // ordinal: TPM_ORD_PCRRead
        0x00, 0x00, 0x00, 0x15,
        // pcrIndex
        0x00, 0x00, 0x00, u8::try_from(pcr_index.0).unwrap(),
    ];

    let mut output = [0; 30];
    tcg.pass_through_to_tpm(&input, &mut output)
        .expect("failed to get PCR value");

    // tag: TPM_TAG_RSP_COMMAND
    assert_eq!(output[0..2], [0x00, 0xc4]);
    // paramSize
    assert_eq!(output[2..6], [0x00, 0x00, 0x00, 0x1e]);
    // returnCode: TPM_SUCCESS
    assert_eq!(output[6..10], [0x00, 0x00, 0x00, 0x00]);

    // The rest of the output is the SHA-1 digest.
    output[10..].try_into().unwrap()
}

fn test_tcg_v1(bt: &BootServices) {
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

    let pcr_index = PcrIndex(8);

    let mut event_buf = [MaybeUninit::uninit(); 256];
    let event = v1::PcrEvent::new_in_buffer(
        &mut event_buf,
        pcr_index,
        EventType::IPL,
        [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13,
        ],
        &[0x14, 0x15, 0x16, 0x17],
    )
    .unwrap();

    // PCR 8 starts at zero.
    assert_eq!(tcg_v1_read_pcr(&mut tcg, pcr_index), [0; 20]);

    tcg.log_event(event).expect("failed to call log_event");

    // `log_event` doesn't extend the PCR, so still zero
    assert_eq!(tcg_v1_read_pcr(&mut tcg, pcr_index), [0; 20]);

    tcg.hash_log_extend_event(event, None)
        .expect("failed to call hash_log_extend_event");

    // The PCR has been extended: `sha1([0; 20], <event digest>)`.
    assert_eq!(
        tcg_v1_read_pcr(&mut tcg, pcr_index),
        [
            0xf8, 0x7c, 0xfc, 0x25, 0xe0, 0x47, 0xab, 0x7f, 0xa1, 0xc1, 0xd2, 0xcc, 0xa2, 0xc7,
            0xff, 0xaa, 0x70, 0x6c, 0xd2, 0x3a,
        ]
    );

    tcg.hash_log_extend_event(event, Some(&[0x18, 0x19, 0x20, 0x21]))
        .expect("failed to call hash_log_extend_event (with data)");

    // The event's digest has been updated with the SHA-1 of `[0x18, 0x19, 0x20, 0x21]`.
    assert_eq!(
        event.digest(),
        [
            0x3a, 0x8a, 0x2a, 0xfc, 0xb1, 0x44, 0x9a, 0xbd, 0x50, 0x24, 0x1b, 0x75, 0xee, 0x49,
            0xba, 0x9b, 0x55, 0xbc, 0xff, 0xff,
        ]
    );

    // The PCR has been extended: `sha1(<previous digest>, <event digest>)`.
    assert_eq!(
        tcg_v1_read_pcr(&mut tcg, pcr_index),
        [
            0x74, 0xc7, 0xe8, 0x5c, 0x4, 0x8, 0x7b, 0x59, 0x7d, 0x81, 0xf4, 0xe7, 0x48, 0x7e, 0x1b,
            0xe8, 0x29, 0x27, 0xc0, 0xb0,
        ]
    );

    // Check the capabilities and feature flags.
    let status = tcg.status_check().expect("failed to call status_check");
    let expected_version = v1::Version {
        major: 1,
        minor: 2,
        rev_major: 0,
        rev_minor: 0,
    };
    assert_eq!(
        status.protocol_capability.structure_version(),
        expected_version,
    );
    assert_eq!(
        status.protocol_capability.protocol_spec_version(),
        expected_version
    );
    assert_eq!(
        status.protocol_capability.hash_algorithm(),
        HashAlgorithm::SHA1
    );
    assert!(status.protocol_capability.tpm_present());
    assert!(!status.protocol_capability.tpm_deactivated());
    assert_eq!(status.feature_flags, 0);

    // Check the event log -- the last two entries should be the ones added
    // above with calls to `hash_log_extend_event`.
    let event_log: Vec<_> = status.event_log.iter().collect();
    let event_second_last = &event_log[event_log.len() - 2];
    assert_eq!(
        event_second_last.digest(),
        [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13
        ]
    );

    let event_last = event_log[event_log.len() - 1];
    assert_eq!(event_last, event);
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
