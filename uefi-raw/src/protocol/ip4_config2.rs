use crate::{guid, Char16, Event, Guid, Status};
use core::ffi::c_void;

// TODO: Move to ip4 module
//
// use crate::protocol::ip4::RouteTable;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RouteTable {
    pub subnet_addr: [u8; 4],
    pub subnet_mask: [u8; 4],
    pub gateway_addr: [u8; 4],
}

newtype_enum! {
    pub enum DataType: i32 => {
        /// The interface information of the communication device this EFI IPv4
        /// Configuration II Protocol instance manages. This type of data is
        /// read only. The corresponding Data is of type
        /// EFI_IP4_CONFIG2_INTERFACE_INFO.
        INTERFACE_INFO = 0,
        /// The general configuration policy for the EFI IPv4 network stack
        /// running on the communication device this EFI IPv4 Configuration II
        /// Protocol instance manages. The policy will affect other
        /// configuration settings. The corresponding Data is of type
        /// EFI_IP4_CONFIG2_POLICY.
        POLICY         = 1,
        /// The station addresses set manually for the EFI IPv4 network stack.
        /// It is only configurable when the policy is Ip4Config2PolicyStatic.
        /// The corresponding Data is of type EFI_IP4_CONFIG2_MANUAL_ADDRESS.
        /// When DataSize is 0 and Data is NULL, the existing configuration is
        /// cleared from the EFI IPv4 Configuration II Protocol instance.
        MANUAL_ADDRESS = 2,
        /// The gateway addresses set manually for the EFI IPv4 network stack
        /// running on the communication device this EFI IPv4 Configuration II
        /// Protocol manages. It is not configurable when the policy is
        /// Ip4Config2PolicyDhcp. The gateway addresses must be unicast IPv4
        /// addresses. The corresponding Data is a pointer to an array of
        /// EFI_IPv4_ADDRESS instances. When DataSize is 0 and Data is NULL, the
        /// existing configuration is cleared from the EFI IPv4 Configuration II
        /// Protocol instance.
        GATEWAY        = 3,
        /// The DNS server list for the EFI IPv4 network stack running on the
        /// communication device this EFI IPv4 Configuration II Protocol
        /// manages. It is not configurable when the policy is
        /// Ip4Config2PolicyDhcp. The DNS server addresses must be unicast IPv4
        /// addresses. The corresponding Data is a pointer to an array of
        /// EFI_IPv4_ADDRESS instances. When DataSize is 0 and Data is NULL, the
        /// existing configuration is cleared from the EFI IPv4 Configuration II
        /// Protocol instance.
        DNS_SERVER     = 4,

        MAX            = 5,
    }
}

/// The EFI_IP4_CONFIG2_INTERFACE_INFO structure describes the operational state
/// of the interface this EFI IPv4 Configuration II Protocol instance manages.
/// This type of data is read-only. When reading, the caller allocated buffer is
/// used to return all of the data, i.e., the first part of the buffer is
/// EFI_IP4_CONFIG2_INTERFACE_INFO and the followings are the route table if
/// present. The caller should NOT free the buffer pointed to by RouteTable, and
/// the caller is only required to free the whole buffer if the data is not
/// needed any more.
#[repr(C)]
pub struct InterfaceInfo {
    pub name: [Char16; 32], // null-terminated
    pub if_type: u8,
    pub hw_addr_size: u32,
    pub hw_addr: [u8; 32],
    pub station_addr: [u8; 4],
    pub subnet_mask: [u8; 4],
    pub route_table_size: u32,
    pub route_table: *mut RouteTable,
}

impl Drop for InterfaceInfo {
    fn drop(&mut self) {
        unsafe {
            core::ptr::drop_in_place(core::slice::from_raw_parts_mut(
                self.route_table,
                self.route_table_size as usize,
            ))
        };
    }
}

newtype_enum! {
    /// The EFI_IP4_CONFIG2_POLICY defines the general configuration policy the
    /// EFI IPv4 Configuration II Protocol supports. The default policy for a
    /// newly detected communication device is beyond the scope of this
    /// document. An implementation might leave it to platform to choose the
    /// default policy.
    ///
    /// The configuration data of type Ip4Config2DataTypeManualAddress,
    /// Ip4Config2DataTypeGateway and Ip4Config2DataTypeDnsServer will be
    /// flushed if the policy is changed from Ip4Config2PolicyStatic to
    /// Ip4Config2PolicyDhcp.
    pub enum Policy: i32 => {
        STATIC = 0,
        DHCP   = 1,
        MAX    = 2,
    }
}

/// The EFI_IP4_CONFIG2_MANUAL_ADDRESS structure is used to set the station
/// address information for the EFI IPv4 network stack manually when the policy
/// is Ip4Config2PolicyStatic.
///
/// The EFI_IP4_CONFIG2_DATA_TYPE includes current supported data types; this
/// specification allows future extension to support more data types.
#[repr(C)]
pub struct ManualAddress {
    pub address: [u8; 4],
    pub subnet_mask: [u8; 4],
}

/// The EFI_IP4_CONFIG2_PROTOCOL provides the mechanism to set and get various
/// types of configurations for the EFI IPv4 network stack.
///
/// The EFI_IP4_CONFIG2_PROTOCOL is designed to be the central repository for
/// the common configurations and the administrator configurable settings for
/// the EFI IPv4 network stack.
///
/// An EFI IPv4 Configuration II Protocol instance will be installed on each
/// communication device that the EFI IPv4 network stack runs on.
#[repr(C)]
pub struct Ip4Config2Protocol {
    /// Set the configuration for the EFI IPv4 network stack running on the
    /// communication device this EFI IPv4 Configuration II Protocol instance
    /// manages.
    ///
    /// This function is used to set the configuration data of type DataType for
    /// the EFI IPv4 network stack running on the communication device this EFI
    /// IPv4 Configuration II Protocol instance manages. The successfully
    /// configured data is valid after system reset or power-off.
    ///
    /// The DataSize is used to calculate the count of structure instances in
    /// the Data for some DataType that multiple structure instances are
    /// allowed.
    ///
    /// This function is always non-blocking. When setting some type of
    /// configuration data, an asynchronous process is invoked to check the
    /// correctness of the data, such as doing address conflict detection on the
    /// manually set local IPv4 address. EFI_NOT_READY is returned immediately
    /// to indicate that such an asynchronous process is invoked and the process
    /// is not finished yet. The caller willing to get the result of the
    /// asynchronous process is required to call RegisterDataNotify() to
    /// register an event on the specified configuration data. Once the event is
    /// signaled, the caller can call GetData() to get back the configuration
    /// data in order to know the result. For other types of configuration data
    /// that do not require an asynchronous configuration process, the result of
    /// the operation is immediately returned.
    pub set_data: unsafe extern "efiapi" fn(
        this: &Self,
        data_type: DataType,
        data_size: usize,
        data: *const c_void,
    ) -> Status,

    /// Get the configuration data for the EFI IPv4 network stack running on the
    /// communication device this EFI IPv4 Configuration II Protocol instance
    /// manages.
    ///
    /// This function returns the configuration data of type DataType for the
    /// EFI IPv4 network stack running on the communication device this EFI IPv4
    /// Configuration II Protocol instance manages.
    ///
    /// The caller is responsible for allocating the buffer used to return the
    /// specified configuration data and the required size will be returned to
    /// the caller if the size of the buffer is too small.
    ///
    /// EFI_NOT_READY is returned if the specified configuration data is not
    /// ready due to an already in progress asynchronous configuration process.
    /// The caller can call RegisterDataNotify() to register an event on the
    /// specified configuration data. Once the asynchronous configuration
    /// process is finished, the event will be signaled and a subsequent
    /// GetData() call will return the specified configuration data.
    pub get_data: unsafe extern "efiapi" fn(
        this: &Self,
        data_type: DataType,
        data_size: *mut usize,
        data: *mut c_void,
    ) -> Status,

    /// Register an event that is to be signaled whenever a configuration
    /// process on the specified configuration data is done.
    ///
    /// This function registers an event that is to be signaled whenever a
    /// configuration process on the specified configuration data is done. An
    /// event can be registered for different DataType simultaneously and the
    /// caller is responsible for determining which type of configuration data
    /// causes the signaling of the event in such case.
    pub register_data_notify:
        unsafe extern "efiapi" fn(this: &Self, data_type: DataType, event: Event) -> Status,

    /// Remove a previously registered event for the specified configuration data.
    ///
    /// This function removes a previously registered event for the specified
    /// configuration data.
    pub unregister_data_notify:
        unsafe extern "efiapi" fn(this: &Self, data_type: DataType, event: Event) -> Status,
}

impl Ip4Config2Protocol {
    pub const GUID: Guid = guid!("5b446ed1-e30b-4faa-871a-3654eca36080");
}
