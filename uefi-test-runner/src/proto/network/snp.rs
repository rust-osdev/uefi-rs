// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::proto::network::{ build_ipv4_udp_packet_smoltcp};
use alloc::string::ToString;
use alloc::vec::Vec;
use core::net::Ipv4Addr;
use smoltcp::wire::EthernetProtocol;
use uefi::boot::{OpenProtocolAttributes, OpenProtocolParams};
use uefi::proto::device_path::text::{AllowShortcuts, DisplayOnly};
use uefi::proto::device_path::DevicePath;
use uefi::proto::network::snp::{InterruptStatus, NetworkState, ReceiveFlags, SimpleNetwork};
use uefi::{boot, Status};

fn compute_ipv4_checksum(header: &[u8]) -> u16 {
    assert_eq!(header.len() % 2, 0);
    let sum = header
        .chunks(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]) as u32)
        .sum::<u32>();

    let carry_add = (sum & 0xFFFF) + (sum >> 16);
    !(carry_add as u16)
}

fn build_ipv4_packet_with_payload(
    src_ip: Ipv4Addr,
    dest_ip: Ipv4Addr,
    payload: [u8; 2],
) -> [u8; 22] {
    let mut packet = [0u8; 22];
    let len = packet.len() as u16;

    // IPv4 header
    // Version = 4, IHL = 5
    packet[0] = 0x45;
    // DSCP/ECN
    packet[1] = 0x00;
    // Total length
    packet[2..4].copy_from_slice(&(len.to_be_bytes()));
    // Identification
    packet[4..6].copy_from_slice(&0u16.to_be_bytes());
    // Flags (DF), Fragment offset
    packet[6..8].copy_from_slice(&0x4000u16.to_be_bytes());
    // TTL
    packet[8] = 0x40;
    // Protocol (UDP)
    packet[9] = 0x11;
    // Checksum placeholder at [10..12]
    packet[12..16].copy_from_slice(&src_ip.octets()); // Source IP
    packet[16..20].copy_from_slice(&dest_ip.octets()); // Destination IP

    // Calculate checksum
    let checksum = compute_ipv4_checksum(&packet[0..20]);
    packet[10..12].copy_from_slice(&checksum.to_be_bytes());

    // Payload
    packet[20] = payload[0];
    packet[21] = payload[1];

    packet
}

pub fn test() {
    info!("Testing the simple network protocol");

    let handles = boot::find_handles::<SimpleNetwork>().unwrap();

    for handle in handles {
        // Buggy firmware; although it should be there, we have to test if the
        // protocol is actually installed.
        let simple_network = match boot::open_protocol_exclusive::<SimpleNetwork>(handle) {
            Ok(snp) => snp,
            Err(e) => {
                log::debug!("Handle {handle:?} doesn't actually support SNP; skipping");
                continue;
            }
        };
        let simple_network_dvp = boot::open_protocol_exclusive::<DevicePath>(handle).unwrap();
        debug!(
            "Testing network device: {}",
            simple_network_dvp
                .to_string(DisplayOnly(false), AllowShortcuts(false))
                .unwrap()
        );

        // Check media
        assert!(
            simple_network.mode().media_present && simple_network.mode().media_present_supported
        );

        // Ensure network is not started yet. If it is started, another test
        // didn't clean up properly.
        assert_eq!(simple_network.mode().state, NetworkState::STOPPED);

        // Check start
        simple_network
            .start()
            .expect("Failed to start Simple Network");

        // Check initialize
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

        assert!(!simple_network
            .get_interrupt_status()
            .unwrap()
            .contains(InterruptStatus::TRANSMIT));

        // Broadcast address (send to self)
        let src_addr = simple_network.mode().current_address;
        let dest_addr = simple_network.mode().broadcast_address;
        const ETHER_TYPE_IPV4: u16 = 0x0800;

        // IPv4 can be arbitrary, not used for routing here.
        let src_ip = Ipv4Addr::new(192, 168, 17, 15);
        let dst_ip = Ipv4Addr::new(192, 168, 17, 2);
        let src_port = 0x5445; // "TE"
        let dst_port = 0x5444; // "TD"
        let payload = 0x1337_u16.to_ne_bytes();
        let ipv4packet = build_ipv4_udp_packet_smoltcp(src_ip.into(), dst_ip.into(), src_port, dst_port, &payload);

        let mut buffer = Vec::<u8>::new();
        /* the implementation will fill the ethernet header correctly */
        buffer.extend_from_slice(&[0; smoltcp::wire::ETHERNET_HEADER_LEN]);
        buffer.extend_from_slice(ipv4packet.as_slice());

        let mut frame = smoltcp::wire::EthernetFrame::new_unchecked(&mut buffer);
        frame.set_src_addr(smoltcp::wire::EthernetAddress::from_bytes(&src_addr.0[..6]));
        frame.set_dst_addr(smoltcp::wire::EthernetAddress::from_bytes(&dest_addr.0[..6]));
        frame.set_ethertype(EthernetProtocol::Ipv4);
        frame.check_len().unwrap();

        log::debug!("frame: {:#x?}", buffer);


        // Send the frame to ourselves
        simple_network
            .transmit(
                simple_network.mode().media_header_size as usize,
                // We send: EthernetFrame(Ipv4Packet(UdpPacket(Payload)))
                &buffer,
                Some(src_addr),
                Some(dest_addr),
                Some(ETHER_TYPE_IPV4),
            )
            .expect("Failed to transmit frame");

        info!("Waiting for the transmit");
        while !simple_network
            .get_interrupt_status()
            .unwrap()
            .contains(InterruptStatus::TRANSMIT)
        {}

        // Attempt to receive a frame
        let mut recv_buffer = [0u8; 1500];

        info!("Waiting for the reception");
        if simple_network.receive(&mut recv_buffer, None, None, None, None)
            == Err(Status::NOT_READY.into())
        {
            boot::stall(1_000_000);

            simple_network
                .receive(&mut recv_buffer, None, None, None, None)
                .unwrap();
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
    }
}
