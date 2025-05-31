// SPDX-License-Identifier: MIT OR Apache-2.0

use core::time::Duration;

use uefi::boot::ScopedProtocol;
use uefi::proto::network::MacAddress;
use uefi::proto::network::snp::{InterruptStatus, ReceiveFlags, SimpleNetwork};
use uefi::{Status, boot};

/// The MAC address configured for the interface.
const EXPECTED_MAC: [u8; 6] = [0x52, 0x54, 0, 0, 0, 0x1];

fn find_network_device() -> Option<ScopedProtocol<SimpleNetwork>> {
    let mut maybe_handle = None;

    let handles = boot::find_handles::<SimpleNetwork>().unwrap_or_default();

    // We iterate over all handles until we found the right network device.
    for handle in handles {
        let Ok(handle) = boot::open_protocol_exclusive::<SimpleNetwork>(handle) else {
            continue;
        };

        // Check media is present
        if !bool::from(handle.mode().media_present_supported)
            || !bool::from(handle.mode().media_present)
        {
            continue;
        }

        // Check MAC address
        let has_mac = handle.mode().current_address.0[0..6] == EXPECTED_MAC
            && handle.mode().permanent_address.0[0..6] == EXPECTED_MAC;
        if !has_mac {
            continue;
        }

        maybe_handle.replace(handle);
    }

    maybe_handle
}

/// This test sends a simple UDP/IP packet to the `EchoService` (created by
/// `cargo xtask run`) and receives its response.
pub fn test() {
    // This test currently depends on the PXE test running first.
    if cfg!(not(feature = "pxe")) {
        return;
    }

    info!("Testing the simple network protocol");

    // The handle to our specific network device, as the test requires also a
    // specific environment. We do not test all possible handles.
    let simple_network = find_network_device().unwrap_or_else(|| panic!(
        "Failed to find SNP handle for network device with MAC address {:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
        EXPECTED_MAC[0],
        EXPECTED_MAC[1],
        EXPECTED_MAC[2],
        EXPECTED_MAC[3],
        EXPECTED_MAC[4],
        EXPECTED_MAC[5]
    ));

    // Check shutdown
    let res = simple_network.shutdown();
    assert!(res == Ok(()) || res == Err(Status::NOT_STARTED.into()));

    // Check stop
    let res = simple_network.stop();
    assert!(res == Ok(()) || res == Err(Status::NOT_STARTED.into()));

    // Check start
    simple_network
        .start()
        .expect("Failed to start Simple Network");

    simple_network
        .initialize(0, 0)
        .expect("Failed to initialize Simple Network");

    // edk2 virtio-net driver does not support statistics, so
    // allow UNSUPPORTED (same for collect_statistics below).
    let res = simple_network.reset_statistics();
    assert!(res == Ok(()) || res == Err(Status::UNSUPPORTED.into()));

    // Reading the interrupt status clears it
    simple_network.get_interrupt_status().unwrap();

    // Set receive filters
    simple_network
        .receive_filters(
            ReceiveFlags::UNICAST | ReceiveFlags::BROADCAST,
            ReceiveFlags::empty(),
            false,
            None,
        )
        .expect("Failed to set receive filters");

    // EthernetFrame(IPv4Packet(UDPPacket(Payload))).
    // The ethernet frame header will be filled by `transmit()`.
    // The UDP packet contains the byte sequence `4, 4, 3, 2, 1`.
    //
    // The packet is sent to the `EchoService` created by
    // `cargo xtask run`. It runs on UDP port 21572.
    let payload = b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
            \x45\x00\
            \x00\x21\
            \x00\x01\
            \x00\x00\
            \x10\
            \x11\
            \x07\x6a\
            \xc0\xa8\x11\x0f\
            \xc0\xa8\x11\x02\
            \x54\x45\
            \x54\x44\
            \x00\x0d\
            \xa9\xe4\
            \x04\x01\x02\x03\x04";

    let dest_addr = MacAddress([0xffu8; 32]);
    assert!(
        !simple_network
            .get_interrupt_status()
            .unwrap()
            .contains(InterruptStatus::TRANSMIT)
    );

    // Send the frame
    simple_network
        .transmit(
            simple_network.mode().media_header_size as usize,
            payload,
            None,
            Some(dest_addr),
            Some(0x0800),
        )
        .expect("Failed to transmit frame");

    info!("Waiting for the transmit");
    while !simple_network
        .get_interrupt_status()
        .unwrap()
        .contains(InterruptStatus::TRANSMIT)
    {}

    // Attempt to receive a frame
    let mut buffer = [0u8; 1500];

    info!("Waiting for the reception");
    if simple_network.receive(&mut buffer, None, None, None, None) == Err(Status::NOT_READY.into())
    {
        boot::stall(Duration::from_secs(1));

        simple_network
            .receive(&mut buffer, None, None, None, None)
            .unwrap();
    }

    // Check payload in UDP packet that was reversed by our EchoService.
    assert_eq!(buffer[42..47], [4, 4, 3, 2, 1]);

    // Get stats
    let res = simple_network.collect_statistics();
    match res {
        Ok(stats) => {
            info!("Stats: {:?}", stats);

            // One frame should have been transmitted and one received
            assert_eq!(stats.tx_total_frames().unwrap(), 1);
            assert_eq!(stats.rx_total_frames().unwrap(), 1);
        }
        Err(e) => {
            if e == Status::UNSUPPORTED.into() {
                info!("Stats: unsupported.");
            } else {
                panic!("{e}");
            }
        }
    }
}
