//! `Ip4 Config2` protocol.

use crate::proto::unsafe_protocol;
use crate::table::boot::{BootServices, EventType, TimerTrigger, Tpl};
use crate::{Error, Event, Result, ResultExt, Status, StatusExt};
use alloc::vec::Vec;
use core::ffi::c_void;
use core::mem;
use uefi_raw::protocol::ip4_config2::{
    DataType, InterfaceInfo, Ip4Config2Protocol, Policy, RouteTable,
};

/// IPv4-only address
type Ip4Address = [u8; 4];

/// Ip4 Config2 protocol
#[unsafe_protocol(Ip4Config2Protocol::GUID,,Ip4Config2Protocol)]
pub struct Ip4Config2<'a> {
    proto: &'a mut Ip4Config2Protocol,
}

impl From<*mut Ip4Config2Protocol> for Ip4Config2<'_> {
    fn from(proto: *mut Ip4Config2Protocol) -> Self {
        Self {
            proto: unsafe { &mut *proto },
        }
    }
}

// Public API
impl Ip4Config2<'_> {
    /// Enable DHCP. This automatically starts the asynchronous handshake and
    /// sets DNS servers based on option codes.
    pub fn enable_dhcp(&mut self) -> Result {
        self.set(DataType::POLICY, Policy::DHCP)
    }

    /// Wait indefinitely for an IP address to be acquired.
    pub fn wait_for_ip(&mut self, bt: &BootServices) -> Result<Ip4Address> {
        // Poll every 100ms
        let poll_interval =
            unsafe { bt.create_event(EventType::TIMER, Tpl::APPLICATION, None, None)? };
        bt.set_timer(&poll_interval, TimerTrigger::Periodic(100_000_000 /* ns */))?;
        let mut events = [unsafe { Event::from_ptr(poll_interval.as_ptr()).unwrap() }];

        loop {
            // Check if IP is already bound
            let info = self.get_interface_info()?;
            if info.station_addr != [0, 0, 0, 0] {
                let _ = bt.close_event(poll_interval);
                return Ok(info.station_addr);
            }

            // Block until notified of an interface update
            let _ = bt.wait_for_event(&mut events).discard_errdata()?;
        }
    }

    fn get_interface_info(&mut self) -> Result<InterfaceInfo> {
        // Try with a route table size of zero
        self.get::<InterfaceInfo>(DataType::INTERFACE_INFO)
            .or_else(|e| {
                if e.status() != Status::BUFFER_TOO_SMALL {
                    return Err(e).discard_errdata();
                }

                // Non-zero number of route table entries need memory allocated
                let mut info_size = *e.data();
                let static_size = mem::size_of::<InterfaceInfo>();
                let dynamic_size = info_size - static_size;
                assert!(dynamic_size % (mem::size_of::<RouteTable>()) == 0,
                "get_data should require a size equal to the size of the InterfaceInfo struct plus a multiple of the RouteTable entries");
                let route_table_size = dynamic_size / mem::size_of::<RouteTable>();
                let mut route_table = mem::ManuallyDrop::new(vec![RouteTable{
                    subnet_addr: [0; 4],
                    subnet_mask: [0; 4],
                    gateway_addr: [0; 4],
                }; route_table_size]);

                // Try again with requested size and pointer to memory
                let mut info: InterfaceInfo = unsafe { mem::zeroed() };
                info.route_table = route_table.as_mut_ptr();
                self.get_with(DataType::INTERFACE_INFO, &mut info_size, &mut info)
                    .discard_errdata()?;
                Ok::<InterfaceInfo, Error<()>>(info)
            })
    }

    /// Wait indefinitely for a DNS servers to be set.
    pub fn wait_for_dns(&mut self, bt: &BootServices) -> Result<Vec<Ip4Address>> {
        // Poll every 100ms
        let poll_interval =
            unsafe { bt.create_event(EventType::TIMER, Tpl::APPLICATION, None, None)? };
        bt.set_timer(&poll_interval, TimerTrigger::Periodic(100_000_000 /* ns */))?;
        let mut events = [unsafe { Event::from_ptr(poll_interval.as_ptr()).unwrap() }];

        loop {
            // Check if IP is already bound
            let servers = self.get_dns_servers()?;
            if servers.len() > 0 {
                let _ = bt.close_event(poll_interval);
                return Ok(servers);
            }

            // Block until notified of an interface update
            let _ = bt.wait_for_event(&mut events).discard_errdata()?;
        }
    }

    fn get_dns_servers(&mut self) -> Result<Vec<Ip4Address>> {
        // Try with a route table size of zero
        self.get::<[Ip4Address; 0]>(DataType::DNS_SERVER)
            .map(|arr| arr.into())
            .or_else(|e| {
                if e.status() != Status::BUFFER_TOO_SMALL {
                    return Err(e).discard_errdata();
                }

                // Try again with requested size and pointer to memory
                let mut arr_size = *e.data();
                assert!(
                    arr_size % (mem::size_of::<Ip4Address>()) == 0,
                    "DNS server array should be a multiple of IPv4 address size"
                );
                let count = arr_size / mem::size_of::<Ip4Address>();
                let mut servers = vec![[0u8, 0, 0, 0]; count];

                self.get_with(DataType::DNS_SERVER, &mut arr_size, servers.as_mut_slice())
                    .discard_errdata()?;
                Ok::<_, Error<()>>(servers)
            })
    }
}

// Wrappers to raw protocol
impl Ip4Config2<'_> {
    /// Set the configuration for the EFI IPv4 network stack running on the
    /// communication device this EFI IPv4 Configuration II Protocol instance
    /// manages.
    fn set<T>(&mut self, data_type: DataType, data: T) -> Result {
        unsafe {
            (self.proto.set_data)(
                &self.proto,
                data_type,
                mem::size_of_val(&data),
                &data as *const _ as *const c_void,
            )
        }
        .to_result()
    }

    /// Get the configuration data for the EFI IPv4 network stack running on the
    /// communication device this EFI IPv4 Configuration II Protocol instance
    /// manages.
    ///
    /// An error includes a non-zero integer as data when the status is
    /// BUFFER_TOO_SMALL to indicate the required size.
    fn get<T>(&mut self, data_type: DataType) -> Result<T, usize> {
        let mut size: usize = mem::size_of::<T>();
        let mut data: T = unsafe { mem::zeroed() };
        self.get_with(data_type, &mut size, &mut data).map(|_| data)
    }

    fn get_with<T: ?Sized>(
        &mut self,
        data_type: DataType,
        data_size: &mut usize,
        data: &mut T,
    ) -> Result<(), usize> {
        unsafe {
            (self.proto.get_data)(
                &self.proto,
                data_type,
                data_size,
                data as *mut _ as *mut c_void,
            )
        }
        .to_result_with_err(|status| match status {
            Status::BUFFER_TOO_SMALL => *data_size,
            _ => 0,
        })
    }

    /// Register an event that is to be signaled whenever a configuration
    /// process on the specified configuration data is done.
    #[allow(unused)]
    fn notify(&mut self, data_type: DataType, bt: &BootServices) -> Result<Event> {
        let event = unsafe { bt.create_event(EventType::empty(), Tpl::APPLICATION, None, None)? };
        unsafe { (self.proto.register_data_notify)(&self.proto, data_type, event.as_ptr()) }
            .to_result_with_val(|| event)
    }

    /// Remove a previously registered event for the specified configuration data.
    #[allow(unused)]
    fn stop_notify(&mut self, data_type: DataType, event: &Event) -> Result {
        unsafe { (self.proto.unregister_data_notify)(&self.proto, data_type, event.as_ptr()) }
            .to_result()
    }
}

impl core::fmt::Debug for Ip4Config2<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let guid = Ip4Config2Protocol::GUID.to_ascii_hex_lower();
        f.write_str(core::str::from_utf8(&guid).unwrap())
    }
}
