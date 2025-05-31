// SPDX-License-Identifier: MIT OR Apache-2.0

use core::ops::DerefMut;
use core::time::Duration;
use smoltcp::wire::{
    ETHERNET_HEADER_LEN, EthernetFrame, IPV4_HEADER_LEN, Ipv4Packet, UDP_HEADER_LEN, UdpPacket,
};
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

            // Ethernet frame header reports proper DST MAC
            let recv_frame = smoltcp::wire::EthernetFrame::new_checked(&buffer).unwrap();
            assert_eq!(
                recv_frame.dst_addr(),
                smoltcp::wire::EthernetAddress::from_bytes(&EXPECTED_MAC)
            );
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

    // High-level payload to send to destination
    let payload = [
        4_u8, /* Number of elements for echo service */
        1, 2, 3, 4,
    ];
    let frame = {
        // IP that was obtained by PXE test running earlier
        // TODO we should make these tests not depend on each other.
        let src_ip = smoltcp::wire::Ipv4Address::new(192, 168, 17, 15);
        let dst_ip = smoltcp::wire::Ipv4Address::new(192, 168, 17, 2);

        let udp_packet_len = UDP_HEADER_LEN + payload.len();
        let ipv4_packet_len = IPV4_HEADER_LEN + udp_packet_len;
        let frame_len = ETHERNET_HEADER_LEN + ipv4_packet_len;

        let mut buffer = vec![0u8; frame_len];

        let mut frame = EthernetFrame::new_unchecked(buffer.as_mut_slice());
        // Ethertype, SRC MAC, and DST MAC will be set by SNP's transmit().

        let ipv4_packet_buffer = &mut frame.payload_mut()[0..ipv4_packet_len];
        let mut ipv4_packet = Ipv4Packet::new_unchecked(ipv4_packet_buffer);
        ipv4_packet.set_header_len(IPV4_HEADER_LEN as u8 /* no extensions */);
        ipv4_packet.set_total_len(ipv4_packet_len as u16);
        ipv4_packet.set_hop_limit(16);
        ipv4_packet.set_next_header(smoltcp::wire::IpProtocol::Udp);
        ipv4_packet.set_dont_frag(true);
        ipv4_packet.set_ident(0x1337);
        ipv4_packet.set_version(4);
        ipv4_packet.set_src_addr(src_ip);
        ipv4_packet.set_dst_addr(dst_ip);

        let mut udp_packet = UdpPacket::new_unchecked(ipv4_packet.payload_mut());
        udp_packet.set_len(udp_packet_len as u16);
        udp_packet.set_src_port(21573);
        udp_packet.set_dst_port(21572);
        udp_packet.payload_mut().copy_from_slice(&payload);
        assert!(udp_packet.check_len().is_ok());

        udp_packet.fill_checksum(&src_ip.into(), &dst_ip.into());
        // Do this last, as it depends on the other checksum.
        ipv4_packet.fill_checksum();
        assert!(ipv4_packet.check_len().is_ok());

        buffer
    };

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
            &frame,
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
    {
        let recv_frame = EthernetFrame::new_checked(&buffer).unwrap();
        let recv_ipv4 = Ipv4Packet::new_checked(recv_frame.payload()).unwrap();
        let udp_packet = UdpPacket::new_checked(recv_ipv4.payload()).unwrap();
        assert_eq!(udp_packet.payload(), &[4, 4, 3, 2, 1]);
    }

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

    // Workaround for OVMF firmware. `stop()` works in CI on x86_64, but not
    // x86 or aarch64.
    if simple_network.mode().state == NetworkState::STARTED {
        simple_network.stop().unwrap();
    }
    simple_network.shutdown().unwrap();
}
