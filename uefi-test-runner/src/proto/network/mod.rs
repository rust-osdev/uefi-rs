// SPDX-License-Identifier: MIT OR Apache-2.0

use alloc::vec::Vec;
use core::net::Ipv4Addr;
use smoltcp::wire::{IpAddress, IpProtocol, Ipv4Address, Ipv4Packet, UdpPacket, IPV4_HEADER_LEN, UDP_HEADER_LEN};

pub fn test() {
    info!("Testing Network protocols");

    // Test idempotence of network stuff
    for _ in 0..2 {
        #[cfg(feature = "pxe")]
        //pxe::test();
        snp::test();
    }
}

#[cfg(feature = "pxe")]
mod pxe;
mod snp;

/// Build an IPv4/UDP packet into a buffer. Must be large enough.
///
/// Returns the full length of the valid data in the buffer.
pub fn build_ipv4_udp_packet_smoltcp(
    src_ip: IpAddress,
    dst_ip: IpAddress,
    src_port: u16,
    dst_port: u16,
    payload: &[u8],
) -> Vec<u8> {
    fn extract_ipv4(ip: IpAddress) -> Ipv4Address {
        match ip {
            IpAddress::Ipv4(v4) => v4,
            //IpAddress::Ipv6(_) => panic!("IPv6 not supported"),
        }
    }

    let total_len = IPV4_HEADER_LEN + UDP_HEADER_LEN + payload.len();
    let mut buf = vec![0; total_len];
    assert!(buf.len() >= total_len, "buffer too small");

    // Write payload first (UDP requires it to be present for checksum)
    let udp_payload_start = IPV4_HEADER_LEN + UDP_HEADER_LEN;
    const PAYLOAD_BEGIN: usize = IPV4_HEADER_LEN + UDP_HEADER_LEN;
    buf[PAYLOAD_BEGIN..PAYLOAD_BEGIN + payload.len()].copy_from_slice(payload);

    // --- Build UDP packet ---
    let mut udp_packet = UdpPacket::new_unchecked(&mut buf[IPV4_HEADER_LEN..udp_payload_start + payload.len()]);
    udp_packet.set_src_port(src_port);
    udp_packet.set_dst_port(dst_port);
    udp_packet.set_len((UDP_HEADER_LEN + payload.len()) as u16);
    udp_packet.fill_checksum(&src_ip, &dst_ip);
    // Check length after length was written to header.
    udp_packet.check_len().unwrap();

    // --- Build IPv4 header ---
    let mut ip_packet = Ipv4Packet::new_unchecked(&mut buf[..total_len]);
    ip_packet.set_version(4);
    ip_packet.set_header_len(5 /* octets: 5 * 4 = 20 bytes */); 
    ip_packet.set_dscp(0);
    ip_packet.set_ecn(0);
    ip_packet.set_total_len(total_len as u16);
    ip_packet.set_ident(0x1234);
    ip_packet.set_dont_frag(true);
    ip_packet.set_hop_limit(64);
    ip_packet.set_next_header(IpProtocol::Udp);
    ip_packet.set_src_addr(extract_ipv4(src_ip));
    ip_packet.set_dst_addr(extract_ipv4(dst_ip));
    ip_packet.fill_checksum();
    // Check length after length was written to header.
    ip_packet.check_len().unwrap();

    buf
}
