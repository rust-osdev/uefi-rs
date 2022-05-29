#![cfg(unix)]

use std::net::UdpSocket;

/// Start a thread that listens on UDP port 21572 and simulates a simple echo
/// service that reverses the incoming messages.
pub fn start_reverse_echo_service() {
    std::thread::spawn(reverse_echo_service);
}

fn reverse_echo_service() {
    let socket = UdpSocket::bind(("127.0.0.1", 21572)).expect("failed to bind to UDP socket");

    let mut buffer = [0; 257];
    loop {
        // Receive a packet.
        let (len, addr) = socket
            .recv_from(&mut buffer)
            .expect("failed to receive packet");
        let buffer = &mut buffer[..len];

        // Extract header information.
        let (payload_len, payload) = buffer.split_first_mut().unwrap();
        assert_eq!(usize::from(*payload_len), payload.len());

        // Simulate processing the data: Reverse the payload.
        payload.reverse();

        // Send a reply.
        socket.send_to(buffer, addr).expect("failed to send packet");
    }
}
