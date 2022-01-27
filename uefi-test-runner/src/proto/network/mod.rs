use core::ptr;
use uefi::prelude::*;

use uefi::proto::network::snp::SimpleNetwork;
use uefi::proto::network::snp::SimpleNetworkStatistics;

pub fn test(bt: &BootServices) {
    info!("Testing Network protocols");

    let handles = bt
        .find_handles::<SimpleNetwork>()
        .expect_success("Failed to get handles for `SimpleNetwork` protocol");

    for handle in handles {
        let nic = bt
            .handle_protocol::<SimpleNetwork>(handle)
            .expect_success("Unknown error");
        let nic = unsafe { &*nic.get() };

        // Check shutdown
        nic.shutdown().expect_success("Failed to shutdown NIC");

        // Check stop
        nic.stop().expect_success("Failed to stop NIC");

        // Check start
        nic.start().expect_success("Failed to start NIC");

        // Check initialize
        nic.initialize(0, 0)
            .expect_success("Failed to initialize NIC");

        // Set receive filters
        nic.receive_filters(0x01 | 0x04 | 0x08, 0, false, 0, core::ptr::null())
            .expect_success("Failed to set receive filters");

        // Check media
        if nic.mode().media_present_supported && !nic.mode().media_present {
            continue;
        }

        let src_addr = nic.mode().current_address;

        // Hand-craft an ARP probe to use as an example frame
        let arp_request: [u8; 42] = [
            // ETHERNET HEADER
            // Destination MAC
            0xff,
            0xff,
            0xff,
            0xff,
            0xff,
            0xff,
            // Source MAC
            src_addr.addr[0],
            src_addr.addr[1],
            src_addr.addr[2],
            src_addr.addr[3],
            src_addr.addr[4],
            src_addr.addr[5],
            // EtherType
            0x08,
            0x06,
            // ARP HEADER
            // Hardware type
            0x00,
            0x01,
            // Protocol type
            0x08,
            0x00,
            // Hardware address length and protocol address length
            0x06,
            0x04,
            // Operation
            0x00,
            0x01,
            // Sender hardware address
            src_addr.addr[0],
            src_addr.addr[1],
            src_addr.addr[2],
            src_addr.addr[3],
            src_addr.addr[4],
            src_addr.addr[5],
            // Sender protocol address
            0x01,
            0x01,
            0x01,
            0x01,
            // Target hardware address
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            // Target protocol address
            0x00,
            0x00,
            0x00,
            0x00,
        ];

        // Send the frame
        nic.transmit(
            0,
            14 + 28, // Ethernet header plus arp
            &arp_request,
            core::ptr::null(),
            core::ptr::null(),
            core::ptr::null(),
        )
        .expect_success("Failed to transmit frame");

        // Get status
        let mut interrupt_status = 0u32;
        let mut tx_buf: *mut u8 = core::ptr::null_mut();
        let tx_buf_ptr: *mut *mut u8 = &mut tx_buf;

        for j in 1..3 {
            nic.get_status(&mut interrupt_status, tx_buf_ptr)
                .expect_success("Failed to get status");
            info!("interrupt_status: {}", interrupt_status);
            info!("tx_buf: {}", unsafe { **tx_buf_ptr });
            bt.stall(5_000);
        }

        // Attempt to receive a frame
        let mut buffer_size = 1500usize;
        let mut buffer = [0u8; 1500];

        // Receive the frame
        for j in 1..3 {
            let res = nic.receive(
                ptr::null_mut(),
                &mut buffer_size as *mut usize,
                &mut buffer,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            );
            info!("receive: {:?}", res);
            bt.stall(5_000_000);
        }

        // Get stats
        let mut stats = SimpleNetworkStatistics::new();
        nic.statistics(false, &mut stats);
        info!("Stats: {:?}", stats);

        // We should probably see some transmit and receive stats
        assert!(stats.tx_total_frames > 0, "tx_total_frames is zero");
        assert!(stats.rx_total_frames > 0, "rx_total_frames is zero");
    }
}
