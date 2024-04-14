use alloc::string::{String, ToString};
use uefi::{Status, StatusExt};
use uefi::prelude::BootServices;
use uefi::proto::unsafe_protocol;
use uefi::Error;
use uefi::table::boot::EventType;
use uefi_raw::Ipv4Address;
use uefi_raw::protocol::network::ip4::Ip4ModeData;
use uefi_raw::protocol::network::tcpv4::TCPv4ConnectionState;
use crate::proto::network::tcpv4::definitions::{TCPv4CompletionToken, TCPv4ConfigData, TCPv4ConnectionMode, TCPv4IoToken, UnmodelledPointer};
use crate::proto::network::tcpv4::managed_event::ManagedEvent;
use crate::proto::network::tcpv4::transmit_data::TCPv4TransmitDataHandle;

#[derive(Debug)]
#[repr(C)]
#[unsafe_protocol("65530BC7-A359-410F-B010-5AADC7EC2B62")]
pub struct TCPv4Protocol {
    get_mode_data_fn: extern "efiapi" fn(
        this: &Self,
        out_connection_state: Option<&mut TCPv4ConnectionState>,
        out_config_data: Option<&mut UnmodelledPointer>,
        out_ip4_mode_data: Option<&mut Ip4ModeData>,
        out_managed_network_config_data: Option<&mut UnmodelledPointer>,
        out_simple_network_mode: Option<&mut UnmodelledPointer>,
    ) -> Status,

    configure_fn: extern "efiapi" fn(
        this: &Self,
        config_data: Option<&TCPv4ConfigData>,
    ) -> Status,

    routes_fn: extern "efiapi" fn(
        this: &Self,
        delete_route: bool,
        subnet_address: &Ipv4Address,
        subnet_mask: &Ipv4Address,
        gateway_address: &Ipv4Address,
    ) -> Status,

    connect_fn: extern "efiapi" fn(
        this: &Self,
        connection_token: &TCPv4CompletionToken,
    ) -> Status,

    accept_fn: extern "efiapi" fn(
        this: &Self,
        listen_token: &UnmodelledPointer,
    ) -> Status,

    pub(crate) transmit_fn: extern "efiapi" fn(
        this: &Self,
        token: &TCPv4IoToken,
    ) -> Status,

    pub receive_fn: extern "efiapi" fn(
        this: &Self,
        token: &TCPv4IoToken,
    ) -> Status,

    close_fn: extern "efiapi" fn(
        this: &Self,
        close_token: &UnmodelledPointer,
    ) -> Status,

    cancel_fn: extern "efiapi" fn(
        this: &Self,
        completion_token: &UnmodelledPointer,
    ) -> Status,

    poll_fn: extern "efiapi" fn(this: &Self) -> Status,
}

impl TCPv4Protocol {
    pub fn reset_stack(&self) {
        // The UEFI specification states that configuring with NULL options "brutally resets" the TCP stack
        (self.configure_fn)(
            self,
            None,
        ).to_result().expect("Failed to reset TCP stack")
    }

    pub fn configure(
        &self,
        bt: &BootServices,
        connection_mode: TCPv4ConnectionMode,
    ) -> uefi::Result<(), String> {
        let configuration = TCPv4ConfigData::new(connection_mode, None);
        // Maximum timeout of 10 seconds
        for _ in 0..10 {
            let result = (self.configure_fn)(
                self,
                Some(&configuration),
            );
            if result == Status::SUCCESS {
                // Configured connection
                return Ok(())
            }
            else if result == Status::NO_MAPPING {
                // DHCP is still running, wait...
                bt.stall(1_000_000);
            }
            else {
                // Error, spin and try again
                bt.stall(1_000_000);
            }
        }
        Err(Error::new(Status::PROTOCOL_ERROR, "Timeout before configuring the connection succeeded.".to_string()))
    }

    pub fn get_tcp_connection_state(&self) -> TCPv4ConnectionState {
        let mut connection_state = core::mem::MaybeUninit::<TCPv4ConnectionState>::uninit();
        let connection_state_ptr = connection_state.as_mut_ptr();
        unsafe {
            (self.get_mode_data_fn)(
                self,
                Some(&mut *connection_state_ptr),
                None,
                None,
                None,
                None,
            ).to_result().expect("Failed to read connection state");
            connection_state.assume_init()
        }
    }

    pub fn get_ipv4_mode_data(&self) -> Ip4ModeData {
        let mut mode_data = core::mem::MaybeUninit::<Ip4ModeData>::uninit();
        let mode_data_ptr = mode_data.as_mut_ptr();
        unsafe {
            (self.get_mode_data_fn)(
                self,
                None,
                None,
                Some(&mut *mode_data_ptr),
                None,
                None,
            ).to_result().expect("Failed to read mode data");
            mode_data.assume_init()
        }
    }

    pub fn connect(
        &mut self,
        bs: &'static BootServices,
    ) {
        let event = ManagedEvent::new(
            bs,
            EventType::NOTIFY_WAIT,
            |_| {},
        );
        let completion_token = TCPv4CompletionToken::new(&event);
        (self.connect_fn)(
            &self,
            &completion_token,
        ).to_result().expect("Failed to call Connect()");
        event.wait();
    }

    pub fn transmit(
        &mut self,
        bs: &'static BootServices,
        data: &[u8],
    ) {
        let event = ManagedEvent::new(
            bs,
            EventType::NOTIFY_WAIT,
            move |_| {
                // TODO(PT): Accept a user-provided closure?
            },
        );

        let tx_data_handle = TCPv4TransmitDataHandle::new(data);
        let tx_data = tx_data_handle.get_data_ref();
        let io_token = TCPv4IoToken::new(&event, Some(&tx_data), None);
        (self.transmit_fn)(
            &self,
            &io_token,
        ).to_result().expect("Failed to transmit");
        event.wait();
    }
}
