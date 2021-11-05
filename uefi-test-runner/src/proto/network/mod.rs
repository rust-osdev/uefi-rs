use core::ptr;
use uefi::prelude::*;
use uefi::proto::network::snp::MacAddress;
use uefi::proto::network::snp::NetworkStatistics;
use uefi::proto::network::snp::Snp;

pub fn test(bt: &BootServices) {
    info!("Testing Network protocols");

    let handles = bt
        .find_handles::<Snp>()
        .expect_success("Failed to get handles for `Snp` protocol");

    for handle in handles {
        let nic = bt.handle_protocol::<Snp>(handle).expect_success("Unknown");
        let nic = unsafe { &*nic.get() };

        // Check start
        nic.start().expect_success("Failed to start NIC");

        // Check initialize
        nic.initialize(4096, 4096).expect_success("Failed to initialize NIC");

        // Prepare to send a frame
        let mut header_size = 14usize;
        let mut buffer_size = 15usize;
        let mut buffer = [
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 10u8,
        ];
        let mut src_addr = nic.mode().current_address();
        let mut dest_addr = nic.mode().current_address();
        let mut protocol = 2048u16;

        // Send a frame with one byte payload
        nic.transmit(
            header_size,
            buffer_size,
            &buffer,
            src_addr as *const MacAddress,
            dest_addr as *const MacAddress,
            protocol,
        )
        .expect_success("Failed to transmit packet");

        // Receive the frame
        nic.receive(
            header_size as *mut usize,
            buffer_size as *mut usize,
            &mut buffer,
            &mut *src_addr,
            &mut *dest_addr,
            protocol as *mut u16,
        )
        .expect_success("Failed to receive packet");
    }
}
