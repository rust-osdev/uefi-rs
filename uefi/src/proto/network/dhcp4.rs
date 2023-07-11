//! `DHCPv4` protocol.

use crate::proto::unsafe_protocol;
use crate::{Result, StatusExt};
use alloc::vec::Vec;
use core::mem;
use uefi_raw::protocol::dhcp4::{ConfigData, Dhcp4Protocol, Event, ModeData};

pub use uefi_raw::protocol::dhcp4::PacketOption;

/// DHCPv4 Protocol
#[unsafe_protocol(Dhcp4Protocol::GUID, Dhcp4Protocol::SERVICE_GUID, Dhcp4Protocol)]
pub struct Dhcp4<'a> {
    proto: &'a mut Dhcp4Protocol,
}

impl From<*mut Dhcp4Protocol> for Dhcp4<'_> {
    fn from(proto: *mut Dhcp4Protocol) -> Self {
        Self {
            proto: unsafe { &mut *proto },
        }
    }
}

impl Dhcp4<'_> {
    /// Configure DHCP options
    pub fn config(&mut self, options: OptionList) -> Result {
        // Configure the protocol
        let mut config: ConfigData = unsafe { mem::zeroed() };
        config.option_count = options.count();
        config.option_list = &mut options.as_ptr();
        unsafe { (self.proto.configure)(&mut self.proto, &config).to_result() }
    }

    /// Try completing a DHCPv4 Discover/Offer/Request/Acknowledge sequence.
    pub fn bind(&mut self) -> Result {
        (self.proto.start)(&mut self.proto, Event::NULL).to_result()
    }

    /// Get the bound IP, returning 0.0.0.0 if no IP is currently bound.
    pub fn bound_ip(&mut self) -> Result<[u8; 4]> {
        self.mode_data().map(|d| d.client_address)
    }

    /// Get the Mode Data for the DHCP client
    fn mode_data(&mut self) -> Result<ModeData> {
        let mut data: ModeData = unsafe { mem::zeroed() };
        unsafe {
            (self.proto.get_mode_data)(&mut self.proto, &mut data)
                .to_result()
                .map(|_| data)
        }
    }
}

/// A list of option codes and associated values.
#[derive(Debug)]
pub struct OptionList {
    // Encoded array of PacketOptions
    buffer: Vec<u8>,

    // Number of PacketOptions encoded
    count: u32,
}

impl OptionList {
    /// Create a new empty list of options.
    pub fn empty() -> Self {
        Self {
            buffer: vec![],
            count: 0,
        }
    }

    /// Add an option to the list.
    pub fn add<const N: usize>(&mut self, option: PacketOption<N>) {
        let PacketOption::<N> {
            op_code,
            length,
            data,
        } = option;

        // Check size invariant
        assert!(
            length as usize == data.len(),
            "packet option [code={:?}]: length {:?} does not match data [len={:?}]",
            op_code,
            length,
            data.len()
        );

        // Manually encode option into byte buffer with zero padding
        self.buffer.push(op_code);
        self.buffer.push(length);
        for byte in data {
            self.buffer.push(byte);
        }

        // Options with no data must still be 3 bytes in size
        if N == 0 {
            self.buffer.push(0x00);
        }

        // Increase option count
        self.count += 1;
    }

    /// Total number of PacketOptions.
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Get a raw pointer to the list, suitable for FFI.
    pub fn as_ptr(&self) -> *const PacketOption<1> {
        self.buffer.as_ptr() as *const PacketOption<1>
    }
}
