// SPDX-License-Identifier: MIT OR Apache-2.0

#![cfg(feature = "alloc")]

//! IP4 Config2 Protocol.

use alloc::vec;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::time::Duration;

use uefi::boot::ScopedProtocol;
use uefi::prelude::*;
use uefi::proto::unsafe_protocol;
use uefi::{print, println};
use uefi_raw::protocol::network::ip4_config2::{
    Ip4Config2DataType, Ip4Config2InterfaceInfo, Ip4Config2Policy, Ip4Config2Protocol,
};
use uefi_raw::Ipv4Address;

/// IP4 Config2 [`Protocol`]. Configure IPv4 networking.
///
/// [`Protocol`]: uefi::proto::Protocol
#[derive(Debug)]
#[unsafe_protocol(Ip4Config2Protocol::GUID)]
pub struct Ip4Config2(pub Ip4Config2Protocol);

impl Ip4Config2 {
    /// Open IP4 Config2 protocol for the given NIC handle.
    pub fn new(nic_handle: Handle) -> uefi::Result<ScopedProtocol<Self>> {
        let protocol;
        unsafe {
            protocol = boot::open_protocol::<Self>(
                boot::OpenProtocolParams {
                    handle: nic_handle,
                    agent: boot::image_handle(),
                    controller: None,
                },
                boot::OpenProtocolAttributes::GetProtocol,
            )?;
        }
        Ok(protocol)
    }

    /// Set configuration data.  It is recommended to type-specific set_* helpers instead of calling this directly.
    pub fn set_data(&mut self, data_type: Ip4Config2DataType, data: &mut [u8]) -> uefi::Result<()> {
        let status = unsafe {
            let data_ptr = data.as_mut_ptr().cast::<c_void>();
            (self.0.set_data)(&mut self.0, data_type, data.len(), data_ptr)
        };
        match status {
            Status::SUCCESS => Ok(()),
            _ => Err(status.into()),
        }
    }

    /// Get configuration data.  It is recommended to type-specific get_* helpers instead of calling this directly.
    pub fn get_data(&mut self, data_type: Ip4Config2DataType) -> uefi::Result<Vec<u8>> {
        let mut data_size = 0;

        // call #1: figure return buffer size
        let status = unsafe {
            let null = core::ptr::null_mut();
            (self.0.get_data)(&mut self.0, data_type, &mut data_size, null)
        };
        if status != Status::BUFFER_TOO_SMALL {
            return Err(status.into());
        }

        // call #2: get data
        let mut data = vec![0; data_size];
        let status = unsafe {
            let data_ptr = data.as_mut_ptr().cast::<c_void>();
            (self.0.get_data)(&mut self.0, data_type, &mut data_size, data_ptr)
        };
        match status {
            Status::SUCCESS => Ok(data),
            _ => Err(status.into()),
        }
    }

    /// Set config policy (static vs. dhcp).
    pub fn set_policy(&mut self, policy: Ip4Config2Policy) -> uefi::Result<()> {
        let mut data: [u8; 4] = policy.0.to_ne_bytes();
        self.set_data(Ip4Config2DataType::POLICY, &mut data)
    }

    /// Get current interface configuration.
    pub fn get_interface_info(&mut self) -> uefi::Result<Ip4Config2InterfaceInfo> {
        let data = self.get_data(Ip4Config2DataType::INTERFACE_INFO)?;
        let info: &Ip4Config2InterfaceInfo =
            unsafe { &*(data.as_ptr().cast::<Ip4Config2InterfaceInfo>()) };
        Ok(Ip4Config2InterfaceInfo {
            name: info.name,
            if_type: info.if_type,
            hw_addr_size: info.hw_addr_size,
            hw_addr: info.hw_addr,
            station_addr: info.station_addr,
            subnet_mask: info.subnet_mask,
            route_table_size: 0,
            route_table: core::ptr::null_mut(),
        })
    }

    fn print_info(info: &Ip4Config2InterfaceInfo) {
        println!(
            "addr v4: {}.{}.{}.{}",
            info.station_addr.0[0],
            info.station_addr.0[1],
            info.station_addr.0[2],
            info.station_addr.0[3],
        );
    }

    /// Bring up network interface.  Does nothing in case the network
    /// is already set up.  Otherwise turns on DHCP and waits until an
    /// IPv4 address has been assigned.  Reports progress on the
    /// console if verbose is set to true.  Returns TIMEOUT error in
    /// case DHCP configuration does not finish within 30 seconds.
    pub fn ifup(&mut self, verbose: bool) -> uefi::Result<()> {
        let no_address = Ipv4Address::default();

        let info = self.get_interface_info()?;
        if info.station_addr != no_address {
            if verbose {
                print!("Network is already up: ");
                Self::print_info(&info);
            }
            return Ok(());
        }

        if verbose {
            print!("DHCP ");
        }
        self.set_policy(Ip4Config2Policy::DHCP)?;

        for _ in 0..30 {
            if verbose {
                print!(".");
            }
            boot::stall(Duration::from_secs(1));
            let info = self.get_interface_info()?;
            if info.station_addr != no_address {
                if verbose {
                    print!(" OK: ");
                    Self::print_info(&info);
                }
                return Ok(());
            }
        }

        Err(Status::TIMEOUT.into())
    }
}
