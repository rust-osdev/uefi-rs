use uefi::{
    prelude::BootServices,
    proto::network::{
        pxe::{BaseCode, DhcpV4Packet, IpFilter, IpFilters, UdpOpFlags},
        IpAddress,
    },
    CStr8,
};

pub fn test(bt: &BootServices) {
    info!("Testing Network protocols");

    if let Ok(handles) = bt.find_handles::<BaseCode>() {
        for handle in handles {
            let mut base_code = bt.open_protocol_exclusive::<BaseCode>(handle).unwrap();

            info!("Starting PXE Base Code");
            base_code
                .start(false)
                .expect("failed to start PXE Base Code");
            base_code
                .dhcp(false)
                .expect("failed to complete a dhcpv4 handshake");

            assert!(base_code.mode().dhcp_ack_received);
            let dhcp_ack: &DhcpV4Packet = base_code.mode().dhcp_ack.as_ref();
            let server_ip = dhcp_ack.bootp_si_addr;
            let server_ip = IpAddress::new_v4(server_ip);

            const EXAMPLE_FILE_NAME: &[u8] = b"example-file.txt\0";
            const EXAMPLE_FILE_CONTENT: &[u8] = b"Hello world!";
            let example_file_name = CStr8::from_bytes_with_nul(EXAMPLE_FILE_NAME).unwrap();

            info!("Getting remote file size");
            let file_size = base_code
                .tftp_get_file_size(&server_ip, example_file_name)
                .expect("failed to query file size");
            assert_eq!(file_size, EXAMPLE_FILE_CONTENT.len() as u64);

            info!("Reading remote file");
            let mut buffer = [0; 512];
            let len = base_code
                .tftp_read_file(&server_ip, example_file_name, Some(&mut buffer))
                .expect("failed to read file");
            let len = usize::try_from(len).unwrap();
            assert_eq!(EXAMPLE_FILE_CONTENT, &buffer[..len]);

            base_code
                .set_ip_filter(&IpFilter::new(IpFilters::STATION_IP, &[]))
                .expect("failed to set IP filter");

            const EXAMPLE_SERVICE_PORT: u16 = 21572;

            info!("Writing UDP packet to example service");

            let payload = [1, 2, 3, 4];
            let header = [payload.len() as u8];
            let mut write_src_port = 0;
            base_code
                .udp_write(
                    UdpOpFlags::ANY_SRC_PORT,
                    &server_ip,
                    EXAMPLE_SERVICE_PORT,
                    None,
                    None,
                    Some(&mut write_src_port),
                    Some(&header),
                    &payload,
                )
                .expect("failed to write UDP packet");

            info!("Reading UDP packet from example service");

            let mut src_ip = server_ip;
            let mut src_port = EXAMPLE_SERVICE_PORT;
            let mut dest_ip = base_code.mode().station_ip;
            let mut dest_port = write_src_port;
            let mut header = [0; 1];
            let mut received = [0; 4];
            base_code
                .udp_read(
                    UdpOpFlags::USE_FILTER,
                    Some(&mut dest_ip),
                    Some(&mut dest_port),
                    Some(&mut src_ip),
                    Some(&mut src_port),
                    Some(&mut header),
                    &mut received,
                )
                .unwrap();

            // Check the header.
            assert_eq!(header[0] as usize, payload.len());
            // Check that we receive the reversed payload.
            received.reverse();
            assert_eq!(payload, received);

            info!("Stopping PXE Base Code");
            base_code.stop().expect("failed to stop PXE Base Code");
        }
    } else {
        warn!("PXE Base Code protocol is not supported");
    }
}
