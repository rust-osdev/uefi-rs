// SPDX-License-Identifier: MIT OR Apache-2.0

use core::ops::DerefMut;
use core::time::Duration;
use uefi::proto::device_path::text::{AllowShortcuts, DisplayOnly};
use uefi::proto::device_path::DevicePath;

use uefi::proto::network::snp::{InterruptStatus, NetworkState, ReceiveFlags, SimpleNetwork};
use uefi::proto::network::MacAddress;
use uefi::{boot, Status};

const ETHERNET_PROTOCOL_IPV4: u16 = 0x0800;

/// Receives the next IPv4 packet and prints corresponding metadata.
fn receive(simple_network: &mut SimpleNetwork, buffer: &mut [u8]) -> uefi::Result<usize> {
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

    res.inspect(|_| {
        debug!("Received:");
        debug!("  src_mac       =  {:x?}", recv_src_mac);
        debug!("  dst_mac       =  {:x?}", recv_dst_mac);
        debug!("  ethernet_proto=0x{:x?}", recv_ethernet_protocol);

        // Ensure that we do not accidentally get an ARP packet, which we
        // do not expect in this test.
        assert_eq!(recv_ethernet_protocol, ETHERNET_PROTOCOL_IPV4);
    })
}

/// This test sends a simple UDP/IP packet to the `EchoService` (created by
/// `cargo xtask run`) and receives its message.
pub fn test() {
    info!("Testing the simple network protocol");

    let handles = boot::find_handles::<SimpleNetwork>().unwrap_or_default();
    for handle in handles {
        let Ok(mut simple_network) = boot::open_protocol_exclusive::<SimpleNetwork>(handle) else {
            continue;
        };
        // Print device path
        {
            let simple_network_dvp = boot::open_protocol_exclusive::<DevicePath>(handle)
                .expect("Should have device path");
            log::info!(
                "Network interface: {}",
                simple_network_dvp
                    .to_string(DisplayOnly(true), AllowShortcuts(true))
                    .unwrap()
            );
        }

        assert_eq!(
            simple_network.mode().state,
            NetworkState::STOPPED,
            "Should be in stopped state"
        );

        // Check media
        if !bool::from(simple_network.mode().media_present_supported)
            || !bool::from(simple_network.mode().media_present)
        {
            continue;
        }

        simple_network
            .start()
            .expect("Network should not be started yet");

        simple_network
            .initialize(0, 0)
            .expect("Network should not be initialized yet");

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

        assert!(!simple_network
            .get_interrupt_status()
            .unwrap()
            .contains(InterruptStatus::TRANSMIT));

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

        simple_network.shutdown().unwrap();
    }
}
