// SPDX-License-Identifier: MIT OR Apache-2.0

use alloc::vec::Vec;
use uefi::boot;
use uefi::proto::tcg::{v1, v2, AlgorithmId, EventType, HashAlgorithm, PcrIndex};

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

fn test_tcg_v1() {
    // Skip the test of the `tpm_v1` feature is not enabled.
    if cfg!(not(feature = "tpm_v1")) {
        return;
    }

    info!("Running TCG v1 test");

    let handle = boot::get_handle_for_protocol::<v1::Tcg>().expect("no TCG handle found");

    let mut tcg =
        boot::open_protocol_exclusive::<v1::Tcg>(handle).expect("failed to open TCG protocol");

    let pcr_index = PcrIndex(8);

    let mut event_buf = [0; 256];
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

/// Get the SHA-1 digest stored in the given PCR index.
fn tcg_v2_read_pcr_8(tcg: &mut v2::Tcg) -> v1::Sha1Digest {
    // Input command block.
    #[rustfmt::skip]
    let input = [
        // tag: TPM_ST_NO_SESSIONS
        0x80, 0x01,
        // commandSize
        0x00, 0x00, 0x00, 0x14,
        // commandCode: TPM_CC_PCR_Read
        0x00, 0x00, 0x01, 0x7e,

        // pcrSelectionIn.count: 1
        0x00, 0x00, 0x00, 0x01,
        // pcrSelectionIn.pcrSelections[0].hash: SHA-1
        0x00, 0x04,
        // pcrSelectionIn.pcrSelections[0].sizeofSelect: 3 bytes
        0x03,
        // pcrSelectionIn.pcrSelections[0].pcrSelect: bitmask array
        // PCR 0-7: don't select
        0x00,
        // PCR 8-15: select only PCR 8
        0x01,
        // PCR 16-23: don't select
        0x00,
    ];

    let mut output = [0; 50];
    tcg.submit_command(&input, &mut output)
        .expect("failed to get PCR value");

    // tag: TPM_ST_NO_SESSIONS
    assert_eq!(output[0..2], [0x80, 0x01]);
    // responseSize
    assert_eq!(output[2..6], [0x00, 0x00, 0x00, 0x32]);
    // responseCode: TPM_RC_SUCCESS
    assert_eq!(output[6..10], [0x00, 0x00, 0x00, 0x00]);
    // Bytes 10..14 are the TPM update counter, ignore.

    // pcrSelectionIn.count: 1
    assert_eq!(output[14..18], [0x00, 0x00, 0x00, 0x01]);
    // pcrSelectionIn.pcrSelections[0].hash: SHA-1
    assert_eq!(output[18..20], [0x00, 0x04]);
    // pcrSelectionIn.pcrSelections[0].sizeofSelect: 3 bytes
    assert_eq!(output[20], 0x03);
    // pcrSelectionIn.pcrSelections[0].pcrSelect: bitmap selecting PCR 8
    assert_eq!(output[21..24], [0x00, 0x01, 0x00]);

    // pcrValues.count: 1
    assert_eq!(output[24..28], [0x00, 0x00, 0x00, 0x01]);
    // pcrValues.digests[0].size: 20 bytes
    assert_eq!(output[28..30], [0x00, 0x14]);

    // The rest of the output is the SHA-1 digest.
    output[30..].try_into().unwrap()
}

pub fn test_tcg_v2() {
    // Skip the test of the `tpm_v2` feature is not enabled.
    if cfg!(not(feature = "tpm_v2")) {
        return;
    }

    info!("Running TCG v2 test");

    let handle = boot::get_handle_for_protocol::<v2::Tcg>().expect("no TCG handle found");

    let mut tcg =
        boot::open_protocol_exclusive::<v2::Tcg>(handle).expect("failed to open TCG protocol");

    let expected_banks =
        HashAlgorithm::SHA1 | HashAlgorithm::SHA256 | HashAlgorithm::SHA384 | HashAlgorithm::SHA512;

    // Check basic TPM info.
    let capability = tcg.get_capability().expect("failed to call get_capability");
    assert_eq!(
        capability.structure_version,
        v2::Version { major: 1, minor: 1 }
    );
    assert_eq!(
        capability.protocol_version,
        v2::Version { major: 1, minor: 1 }
    );
    assert_eq!(capability.hash_algorithm_bitmap, expected_banks,);
    assert_eq!(
        capability.supported_event_logs,
        v2::EventLogFormat::TCG_1_2 | v2::EventLogFormat::TCG_2
    );
    assert!(capability.tpm_present());
    assert_eq!(capability.max_command_size, 4096);
    assert_eq!(capability.max_response_size, 4096);
    assert_eq!(capability.manufacturer_id, 0x4d4249);
    assert_eq!(capability.number_of_pcr_banks, 4);
    assert_eq!(capability.active_pcr_banks, expected_banks);

    // Check the active PCR banks.
    assert_eq!(
        tcg.get_active_pcr_banks()
            .expect("get_active_pcr_banks failed"),
        expected_banks,
    );

    // Set the active PCR banks. This should succeed, but won't have any effect
    // since we're not rebooting the system.
    tcg.set_active_pcr_banks(HashAlgorithm::SHA256)
        .expect("set_active_pcr_banks failed");

    // Check that there was no attempt to change the active banks in the
    // previous boot.
    assert!(tcg
        .get_result_of_set_active_pcr_banks()
        .expect("get_result_of_set_active_pcr_banks failed")
        .is_none());

    // PCR 8 is initially zero.
    assert_eq!(tcg_v2_read_pcr_8(&mut tcg), [0; 20]);

    // Create a PCR event.
    let pcr_index = PcrIndex(8);
    let mut event_buf = [0; 256];
    let event_data = [0x12, 0x13, 0x14, 0x15];
    let data_to_hash = b"some-data";
    let event =
        v2::PcrEventInputs::new_in_buffer(&mut event_buf, pcr_index, EventType::IPL, &event_data)
            .unwrap();

    // Extend a PCR and add the event to the log.
    tcg.hash_log_extend_event(v2::HashLogExtendEventFlags::empty(), data_to_hash, event)
        .unwrap();

    // Hashes of `data_to_hash`.
    #[rustfmt::skip]
    let expected_hash_sha1 = [
        0x2e, 0x75, 0xc6, 0x98, 0x23, 0x96, 0x8a, 0x24, 0x4f, 0x0c,
        0x55, 0x59, 0xbb, 0x46, 0x8f, 0x36, 0x5f, 0x12, 0x11, 0xb6,
    ];
    #[rustfmt::skip]
    let expected_hash_sha256 = [
        0x93, 0x32, 0xd9, 0x4d, 0x5e, 0xe6, 0x9a, 0xd1,
        0x7d, 0x31, 0x0e, 0x62, 0xcd, 0x10, 0x1d, 0x70,
        0xf5, 0x78, 0x02, 0x4f, 0xd5, 0xe8, 0xd1, 0x64,
        0x7f, 0x80, 0x73, 0xf8, 0x86, 0xc8, 0x94, 0xe1,
    ];
    #[rustfmt::skip]
    let expected_hash_sha384 = [
        0xce, 0x4c, 0xbb, 0x09, 0x78, 0x37, 0x49, 0xbe,
        0xff, 0xc7, 0x17, 0x84, 0x5d, 0x27, 0x69, 0xae,
        0xd1, 0xe7, 0x23, 0x02, 0xdc, 0xeb, 0x95, 0xaf,
        0x34, 0xe7, 0xb4, 0xeb, 0xb9, 0xa8, 0x50, 0x25,
        0xc8, 0x40, 0xc1, 0xca, 0xf8, 0x9e, 0xb7, 0x36,
        0x23, 0x73, 0x09, 0x99, 0x82, 0x10, 0x82, 0x80,
    ];
    #[rustfmt::skip]
    let expected_hash_sha512 = [
        0xe1, 0xc4, 0xfc, 0x67, 0xf1, 0x90, 0x9e, 0x35,
        0x08, 0x3c, 0xc5, 0x30, 0x9f, 0xcb, 0xa3, 0x6d,
        0x27, 0x43, 0x33, 0xa3, 0xc4, 0x00, 0x9a, 0x94,
        0xa9, 0x70, 0x52, 0x73, 0xe4, 0x1f, 0xc8, 0x8b,
        0x61, 0x89, 0xad, 0x15, 0x75, 0x51, 0xe3, 0xd3,
        0x9d, 0x1d, 0xaa, 0x44, 0x12, 0x26, 0x4d, 0x13,
        0x12, 0x0b, 0x67, 0x13, 0xc9, 0x9d, 0x3b, 0xe4,
        0xd6, 0x4c, 0x7d, 0xf4, 0xea, 0x7a, 0x4c, 0x7b,
    ];

    // Get the v1 log, and validate the last entry is the one we just added above.
    let log = tcg.get_event_log_v1().unwrap();
    assert!(!log.is_truncated());
    let entry = log.iter().last().unwrap();
    assert_eq!(entry.pcr_index(), pcr_index);
    assert_eq!(entry.event_type(), EventType::IPL);
    assert_eq!(entry.event_data(), event_data);
    #[rustfmt::skip]
    assert_eq!(entry.digest(), expected_hash_sha1);

    // Get the v2 log, and validate the last entry is the one we just added above.
    let log = tcg.get_event_log_v2().unwrap();
    assert!(!log.is_truncated());
    let entry = log.iter().last().unwrap();
    assert_eq!(entry.pcr_index(), pcr_index);
    assert_eq!(entry.event_type(), EventType::IPL);
    assert_eq!(entry.event_data(), event_data);
    assert_eq!(
        entry.digests().into_iter().collect::<Vec<_>>(),
        [
            (AlgorithmId::SHA1, expected_hash_sha1.as_slice()),
            (AlgorithmId::SHA256, expected_hash_sha256.as_slice()),
            (AlgorithmId::SHA384, expected_hash_sha384.as_slice()),
            (AlgorithmId::SHA512, expected_hash_sha512.as_slice()),
        ]
    );

    // PCR 8 has been extended: `sha1([0; 20], sha1("some-data"))`.
    assert_eq!(
        tcg_v2_read_pcr_8(&mut tcg),
        [
            0x16, 0x53, 0x7d, 0xaa, 0x5d, 0xbd, 0xa8, 0x45, 0xe3, 0x30, 0x9e, 0x40, 0xe8, 0x74,
            0xd1, 0x50, 0x64, 0x73, 0x2f, 0x87,
        ]
    );
}

pub fn test() {
    test_tcg_v1();
    test_tcg_v2();
}
