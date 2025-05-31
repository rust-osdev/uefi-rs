// SPDX-License-Identifier: MIT OR Apache-2.0

use core::ops::DerefMut;
use core::time::Duration;

use uefi::boot::ScopedProtocol;
use uefi::proto::network::MacAddress;
use uefi::proto::network::snp::{InterruptStatus, ReceiveFlags, SimpleNetwork};
use uefi::{Status, boot};
use uefi_raw::protocol::network::snp::NetworkState;

/// The MAC address configured for the interface.
const EXPECTED_MAC: [u8; 6] = [0x52, 0x54, 0, 0, 0, 0x1];
const ETHERNET_PROTOCOL_IPV4: u16 = 0x0800;

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

/// Receives the next IPv4 packet and prints corresponding metadata.
///
/// Returns the length of the response.
fn receive(simple_network: &mut SimpleNetwork, buffer: &mut [u8]) -> uefi::Result<usize> {
    // Wait for a bit to ensure that the previous packet has been processed.
    boot::stall(Duration::from_millis(500));

    let mut recv_src_mac = MacAddress([0; 32]);
    let mut recv_dst_mac = MacAddress([0; 32]);
    let mut recv_ethernet_protocol = 0;

    let res = simple_network.receive(
        buffer,
        None,
        Some(&mut recv_src_mac),
        Some(&mut recv_dst_mac),
        Some(&mut recv_ethernet_protocol),
    );

    // To simplify debugging when receive an unexpected packet, we print the
    // necessary info. This is especially useful if an unexpected IPv4 or ARP
    // packet is received, which can easily happen when fiddling around with
    // this test.
    res.inspect(|_| {
        debug!("Received:");
        debug!("  src_mac       =  {:x?}", &recv_src_mac.0[0..6]);
        debug!("  dst_mac       =  {:x?}", &recv_dst_mac.0[0..6]);
        debug!("  ethernet_proto=0x{:x?}", recv_ethernet_protocol);

        // Assert the ethernet frame was sent to the expected interface.
        {
            // UEFI reports proper DST MAC
            assert_eq!(recv_dst_mac.0[0..6], EXPECTED_MAC);
        }

        // Ensure that we do not accidentally get an ARP packet, which we
        // do not expect in this test.
        assert_eq!(recv_ethernet_protocol, ETHERNET_PROTOCOL_IPV4)
    })
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
    let mut simple_network = find_network_device().unwrap_or_else(|| panic!(
        "Failed to find SNP handle for network device with MAC address {:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
        EXPECTED_MAC[0],
        EXPECTED_MAC[1],
        EXPECTED_MAC[2],
        EXPECTED_MAC[3],
        EXPECTED_MAC[4],
        EXPECTED_MAC[5]
    ));

    assert_eq!(
        simple_network.mode().state,
        NetworkState::STOPPED,
        "Should be in stopped state"
    );

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
            Some(simple_network.mode().broadcast_address),
            Some(ETHERNET_PROTOCOL_IPV4),
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
    let n = receive(simple_network.deref_mut(), &mut buffer).unwrap();
    debug!("Reply has {n} bytes");

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

    simple_network.stop().unwrap();
    simple_network.shutdown().unwrap();
}
