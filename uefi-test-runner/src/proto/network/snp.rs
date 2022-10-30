use uefi::prelude::BootServices;
use uefi::proto::network::snp::SimpleNetwork;
use uefi::proto::network::MacAddress;


pub fn test(bt: &BootServices) {
    info!("Testing the simple network protocol");

        let handles = bt
        .find_handles::<SimpleNetwork>()
        .expect("Failed to get handles for `SimpleNetwork` protocol");

    for handle in handles {

        let simple_network = bt.open_protocol_exclusive::<SimpleNetwork>(handle);
        if simple_network.is_err() { continue; }
        let simple_network = simple_network.unwrap();

        // Check shutdown
        simple_network.shutdown().expect("Failed to shutdown Simple Network");

        // Check stop
        simple_network.stop().expect("Failed to stop Simple Network");

        // Check start
        simple_network.start().expect("Failed to start Simple Network");

        // Check initialize
        simple_network.initialize(None, None)
            .expect("Failed to initialize Simple Network");

        simple_network.reset_statistics().unwrap();

        // Reading the interrupt status clears it
        simple_network.get_interrupt_status().unwrap();

        // Set receive filters
        simple_network.receive_filters(0x01 | 0x02 | 0x04 | 0x08 | 0x10, 0, false, None, None)
            .expect("Failed to set receive filters");

        // Check media
        if !simple_network.mode().media_present_supported || !simple_network.mode().media_present {
            continue;
        }

        let payload = &[0u8; 46];

        let dest_addr = MacAddress([0xffu8;32]);
        assert!(!simple_network.get_interrupt_status().unwrap().transmit_interrupt());
        // Send the frame
        simple_network.transmit(
            simple_network.mode().media_header_size as usize,
            payload,
            None,
            Some(&dest_addr),
            Some(&0x0800),
        )
        .expect("Failed to transmit frame");

        info!("Waiting for the transmit");
        while !simple_network.get_interrupt_status().unwrap().transmit_interrupt() {}

        // Attempt to receive a frame
        let mut buffer = [0u8; 1500];
    
        let mut count = 0;
            
        info!("Waiting for the reception");
        while count < 1_000 {
            let result = simple_network.receive(
                &mut buffer,
                None,
                None,
                None,
                None
            );
            if result.is_ok() { break; }
            count += 1;
        }

        // Get stats
        let stats = simple_network.collect_statistics().expect("Failed to collect statistics");
        info!("Stats: {:?}", stats);

        // One frame should have been transmitted and one received
        assert_eq!(stats.tx_total_frames().unwrap(), 1);
        assert_eq!(stats.rx_total_frames().unwrap(), 1);
    }
}
