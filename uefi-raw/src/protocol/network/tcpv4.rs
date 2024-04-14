use crate::{Handle, Status};

#[derive(Debug)]
#[repr(C)]
pub struct TCPv4Option {
    receive_buffer_size: u32,
    send_buffer_size: u32,
    max_syn_back_log: u32,
    connection_timeout: u32,
    data_retries: u32,
    fin_timeout: u32,
    time_wait_timeout: u32,
    keep_alive_probes: u32,
    keep_alive_time: u32,
    keep_alive_interval: u32,
    enable_nagle: bool,
    enable_time_stamp: bool,
    enable_window_scaling: bool,
    enable_selective_ack: bool,
    enable_path_mtu_discovery: bool,
}

#[derive(Debug)]
#[repr(C)]
pub enum TCPv4ConnectionState {
    Closed = 0,
    Listen = 1,
    SynSent = 2,
    SynReceived = 3,
    Established = 4,
    FinWait1 = 5,
    FinWait2 = 6,
    Closing = 7,
    TimeWait = 8,
    CloseWait = 9,
    LastAck = 10,
}

#[derive(Debug)]
#[repr(C)]
#[unsafe_protocol("00720665-67EB-4a99-BAF7-D3C33A1C7CC9")]
pub struct TCPv4ServiceBindingProtocol {
    pub(crate) create_child: extern "efiapi" fn(
        this: &Self,
        out_child_handle: &mut Handle,
    ) -> Status,

    destroy_child: extern "efiapi" fn(
        this: &Self,
        child_handle: Handle,
    ) -> Status,
}

