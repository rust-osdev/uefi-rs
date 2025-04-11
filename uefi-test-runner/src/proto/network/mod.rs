// SPDX-License-Identifier: MIT OR Apache-2.0

use alloc::vec::Vec;
use core::net::Ipv4Addr;

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

/// Computes the standard IPv4 or UDP checksum (one's complement)
fn compute_ipv4_checksum(data: &[u8]) -> u16 {
    let mut sum = 0u32;

    for chunk in data.chunks(2) {
        let word = if chunk.len() == 2 {
            u16::from_be_bytes([chunk[0], chunk[1]]) as u32
        } else {
            (chunk[0] as u32) << 8
        };
        sum = sum.wrapping_add(word);
    }

    // Fold carry
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    !sum as u16
}

/// Builds the UDP pseudo-header used for UDP checksum calculation
fn build_udp_pseudo_header(src_ip: &Ipv4Addr, dst_ip: &Ipv4Addr, udp_segment: &[u8]) -> Vec<u8> {
    let mut pseudo = Vec::with_capacity(12 + udp_segment.len());

    // Pseudo-header: src IP, dst IP, zero, protocol, UDP length
    pseudo.extend_from_slice(&src_ip.octets());
    pseudo.extend_from_slice(&dst_ip.octets());
    pseudo.push(0);
    pseudo.push(0x11); // Protocol = UDP
    pseudo.extend_from_slice(&(udp_segment.len() as u16).to_be_bytes());

    pseudo.extend_from_slice(udp_segment);

    if pseudo.len() % 2 != 0 {
        pseudo.push(0); // Pad to even length
    }

    pseudo
}

fn build_ipv4_udp_packet(
    src_ip: Ipv4Addr,
    dst_ip: Ipv4Addr,
    src_port: u16,
    dst_port: u16,
    payload: &[u8],
) -> Vec<u8> {
    let ip_header_len = 20;
    let udp_header_len = 8;
    let total_len = ip_header_len + udp_header_len + payload.len();

    let mut packet = Vec::with_capacity(total_len);

    // === IPv4 Header ===
    packet.push(0x45); // Version (4) + IHL (5)
    packet.push(0x00); // DSCP/ECN
    packet.extend_from_slice(&(total_len as u16).to_be_bytes()); // Total Length
    packet.extend_from_slice(&0x0001u16.to_be_bytes()); // Identification
    packet.extend_from_slice(&0x0000u16.to_be_bytes()); // Flags/Fragment offset
    packet.push(0x40); // TTL
    packet.push(0x11); // Protocol = UDP (17)
    packet.extend_from_slice(&[0x00, 0x00]); // Checksum placeholder
    packet.extend_from_slice(&src_ip.octets()); // Source IP
    packet.extend_from_slice(&dst_ip.octets()); // Destination IP

    // Compute IPv4 header checksum
    let ip_checksum = compute_ipv4_checksum(&packet[..20]);
    packet[10..12].copy_from_slice(&ip_checksum.to_be_bytes());

    // === UDP Header ===
    packet.extend_from_slice(&src_port.to_be_bytes()); // Source port
    packet.extend_from_slice(&dst_port.to_be_bytes()); // Destination port
    let udp_len = (udp_header_len + payload.len()) as u16;
    packet.extend_from_slice(&udp_len.to_be_bytes()); // UDP length
    packet.extend_from_slice(&[0x00, 0x00]); // UDP checksum placeholder

    // === Payload ===
    packet.extend_from_slice(payload);

    // === UDP Checksum ===
    let udp_offset = 20;
    let udp_packet = &packet[udp_offset..];
    let pseudo_header = build_udp_pseudo_header(&src_ip, &dst_ip, udp_packet);
    let udp_checksum = compute_ipv4_checksum(&pseudo_header);
    packet[udp_offset + 6..udp_offset + 8].copy_from_slice(&udp_checksum.to_be_bytes());

    packet
}
