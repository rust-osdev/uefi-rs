use uefi::prelude::BootServices;
use uefi::proto::network::snp::{InterruptStatus, ReceiveFlags, SimpleNetwork};
use uefi::proto::network::MacAddress;
use uefi::Status;

pub fn test(bt: &BootServices) {
    info!("Testing the simple network protocol");

    let handles = bt.find_handles::<SimpleNetwork>().unwrap_or_default();

    for handle in handles {
        let simple_network = bt.open_protocol_exclusive::<SimpleNetwork>(handle);
        if simple_network.is_err() {
            continue;
        }
        let simple_network = simple_network.unwrap();

        // Check shutdown
        simple_network
            .shutdown()
            .expect("Failed to shutdown Simple Network");

        // Check stop
        simple_network
            .stop()
            .expect("Failed to stop Simple Network");

        // Check start
        simple_network
            .start()
            .expect("Failed to start Simple Network");

        // Check initialize
        simple_network
            .initialize(0, 0)
            .expect("Failed to initialize Simple Network");

        simple_network.reset_statistics().unwrap();

        // Reading the interrupt status clears it
        simple_network.get_interrupt_status().unwrap();

        // Set receive filters
        simple_network
            .receive_filters(
                ReceiveFlags::UNICAST | ReceiveFlags::MULTICAST | ReceiveFlags::BROADCAST,
                ReceiveFlags::empty(),
                false,
                None,
            )
            .expect("Failed to set receive filters");

        // Check media
        if !simple_network.mode().media_present_supported || !simple_network.mode().media_present {
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
            bt.stall(1_000_000);

            simple_network
                .receive(&mut buffer, None, None, None, None)
                .unwrap();
        }

        assert_eq!(buffer[42..47], [4, 4, 3, 2, 1]);

        // Get stats
        let stats = simple_network
            .collect_statistics()
            .expect("Failed to collect statistics");
        info!("Stats: {:?}", stats);

        // One frame should have been transmitted and one received
        assert_eq!(stats.tx_total_frames().unwrap(), 1);
        assert_eq!(stats.rx_total_frames().unwrap(), 1);
    }
}
