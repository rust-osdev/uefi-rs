// SPDX-License-Identifier: MIT OR Apache-2.0

use core::time::Duration;

use uefi::proto::network::snp::{InterruptStatus, ReceiveFlags, SimpleNetwork};
use uefi::proto::network::MacAddress;
use uefi::{boot, Status};

pub fn test() {
    info!("Testing the simple network protocol");

    let handles = boot::find_handles::<SimpleNetwork>().unwrap_or_default();

    for handle in handles {
        let simple_network = boot::open_protocol_exclusive::<SimpleNetwork>(handle);
        if simple_network.is_err() {
            continue;
        }
        let simple_network = simple_network.unwrap();

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

        // Check media
        if !bool::from(simple_network.mode().media_present_supported)
            || !bool::from(simple_network.mode().media_present)
        {
            continue;
        }

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
        if simple_network.receive(&mut buffer, None, None, None, None)
            == Err(Status::NOT_READY.into())
        {
            boot::stall(Duration::from_secs(1));

            simple_network
                .receive(&mut buffer, None, None, None, None)
                .unwrap();
        }

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
}
