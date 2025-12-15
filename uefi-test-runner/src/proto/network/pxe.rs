// SPDX-License-Identifier: MIT OR Apache-2.0

use core::net::{IpAddr, Ipv4Addr};
use uefi::proto::network::pxe::{BaseCode, DhcpV4Packet, IpFilter, IpFilters, UdpOpFlags};
use uefi::{CStr8, boot};

pub fn test() {
    // Skip the test if the `pxe` feature is not enabled.
    if cfg!(not(feature = "pxe")) {
        return;
    }

    info!("Testing The PXE base code protocol");

    let handles = boot::find_handles::<BaseCode>().expect("failed to get PXE base code handles");
    for handle in handles {
        let mut base_code = boot::open_protocol_exclusive::<BaseCode>(handle).unwrap();

        info!("Starting PXE Base Code");
        base_code
            .start(false)
            .expect("failed to start PXE Base Code");
        base_code
            .dhcp(false)
            .expect("failed to complete a dhcpv4 handshake");

        assert!(base_code.mode().dhcp_ack_received());
        let dhcp_ack: &DhcpV4Packet = base_code.mode().dhcp_ack().as_ref();

        info!("DHCP: Server IP: {:?}", dhcp_ack.bootp_si_addr);
        info!("DHCP: Client IP: {:?}", dhcp_ack.bootp_yi_addr);

        let server_ip = IpAddr::V4(Ipv4Addr::from(dhcp_ack.bootp_si_addr));

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

        // Used as buffers
        let mut src_ip = IpAddr::V4(Ipv4Addr::from([0; 4]));
        let mut src_port = 0;
        let mut dest_ip = IpAddr::V4(Ipv4Addr::from([0; 4]));
        let mut dest_port = 0;
        let mut header = [0; 1];
        let mut received = [0; 4];

        // The Windows CI job sometimes fails the read with a timeout error;
        // retry a few times before giving up.
        let mut read_result = Ok(0);
        for i in 0..5 {
            read_result = base_code.udp_read(
                // We expect exactly one packet but accept all to catch
                // unexpected network traffic.
                UdpOpFlags::ANY_SRC_PORT
                    | UdpOpFlags::ANY_SRC_IP
                    | UdpOpFlags::ANY_DEST_PORT
                    | UdpOpFlags::ANY_DEST_IP,
                Some(&mut dest_ip),
                Some(&mut dest_port),
                Some(&mut src_ip),
                Some(&mut src_port),
                Some(&mut header),
                &mut received,
            );
            if read_result.is_ok() {
                break;
            }

            info!("Read attempt {i} failed: {read_result:?}");
        }
        read_result.unwrap();

        // Check that we indeed received the expected packet.
        assert_eq!(dest_ip, base_code.mode().station_ip());
        assert_eq!(src_ip, server_ip);
        assert_eq!(src_port, EXAMPLE_SERVICE_PORT);
        // We don't know the dst port here, as it is dynamically handled
        // by QEMU/the NIC.
        debug!("dest UDP port: {dest_port}");

        // Check the header.
        assert_eq!(header[0] as usize, payload.len());
        // Check that we receive the reversed payload.
        received.reverse();
        assert_eq!(payload, received);

        info!("Stopping PXE Base Code");
        base_code.stop().expect("failed to stop PXE Base Code");
    }
}
