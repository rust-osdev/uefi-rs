use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Run a simple echo service that listens on UDP port 21572 and
/// reverses the incoming messages.
pub struct EchoService {
    stop_requested: Arc<Mutex<bool>>,

    // `JoinHandle::join` consumes the handle, so put it in an option so
    // that we can call `join` in `drop`.
    join_handle: Option<JoinHandle<()>>,
}

impl Drop for EchoService {
    fn drop(&mut self) {
        self.stop();
        self.join_handle
            .take()
            .unwrap()
            .join()
            .expect("failed to join echo service thread");
    }
}

impl EchoService {
    /// Start the server.
    pub fn start() -> Self {
        let stop_requested = Arc::new(Mutex::new(false));
        let stop_requested_copy = stop_requested.clone();
        let join_handle = thread::spawn(|| reverse_echo_service(stop_requested_copy));
        Self {
            stop_requested,
            join_handle: Some(join_handle),
        }
    }

    /// Request that the server stop.
    pub fn stop(&self) {
        let mut guard = self.stop_requested.lock().unwrap();
        *guard = true;
    }
}

fn reverse_echo_service(stop_requested: Arc<Mutex<bool>>) {
    let socket = UdpSocket::bind(("127.0.0.1", 21572)).expect("failed to bind to UDP socket");

    // Set a timeout so that the service can periodically check if a
    // stop has been requested.
    socket
        .set_read_timeout(Some(Duration::from_millis(100)))
        .expect("failed to set read timeout");

    let mut buffer = [0; 257];
    loop {
        if *stop_requested.lock().unwrap() {
            break;
        }

        // Receive a packet.
        let (len, addr) = match socket.recv_from(&mut buffer) {
            Ok((len, addr)) => (len, addr),
            Err(_) => continue,
        };
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
