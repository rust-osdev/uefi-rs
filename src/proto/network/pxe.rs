//! PXE Base Code protocol.

use core::{
    ffi::c_void,
    iter::from_fn,
    ptr::{null, null_mut},
};

use bitflags::bitflags;
use uefi_macros::{unsafe_guid, Protocol};

use crate::{CStr8, Char8, Result, Status};

use super::{IpAddress, MacAddress};

/// PXE Base Code protocol
#[repr(C)]
#[unsafe_guid("03c4e603-ac28-11d3-9a2d-0090273fc14d")]
#[derive(Protocol)]
#[allow(clippy::type_complexity)]
pub struct BaseCode {
    revision: u64,
    start: extern "efiapi" fn(this: &Self, use_ipv6: bool) -> Status,
    stop: extern "efiapi" fn(this: &Self) -> Status,
    dhcp: extern "efiapi" fn(this: &Self, sort_offers: bool) -> Status,
    discover: extern "efiapi" fn(
        this: &Self,
        ty: BootstrapType,
        layer: &mut u16,
        use_bis: bool,
        info: Option<*const DiscoverInfo<[Server; 0]>>,
    ) -> Status,
    mtftp: unsafe extern "efiapi" fn(
        this: &Self,
        operation: TftpOpcode,
        buffer: *mut c_void,
        overwrite: bool,
        buffer_size: &mut u64,
        block_size: Option<&usize>,
        server_ip: &IpAddress,
        filename: *const Char8,
        info: Option<&MtftpInfo>,
        dont_use_buffer: bool,
    ) -> Status,
    udp_write: unsafe extern "efiapi" fn(
        this: &Self,
        op_flags: UdpOpFlags,
        dest_ip: &IpAddress,
        dest_port: &u16,
        gateway_ip: Option<&IpAddress>,
        src_ip: Option<&IpAddress>,
        src_port: Option<&mut u16>,
        header_size: Option<&usize>,
        header_ptr: *const c_void,
        buffer_size: &usize,
        buffer_ptr: *const c_void,
    ) -> Status,
    udp_read: unsafe extern "efiapi" fn(
        this: &Self,
        op_flags: UdpOpFlags,
        dest_ip: Option<&mut IpAddress>,
        dest_port: Option<&mut u16>,
        src_ip: Option<&mut IpAddress>,
        src_port: Option<&mut u16>,
        header_size: Option<&usize>,
        header_ptr: *mut c_void,
        buffer_size: &mut usize,
        buffer_ptr: *mut c_void,
    ) -> Status,
    set_ip_filter: extern "efiapi" fn(this: &Self, new_filter: &IpFilter) -> Status,
    arp: extern "efiapi" fn(
        this: &Self,
        ip_addr: &IpAddress,
        mac_addr: Option<&mut MacAddress>,
    ) -> Status,
    set_parameters: extern "efiapi" fn(
        this: &Self,
        new_auto_arp: Option<&bool>,
        new_send_guid: Option<&bool>,
        new_ttl: Option<&u8>,
        new_tos: Option<&u8>,
        new_make_callback: Option<&bool>,
    ) -> Status,
    set_station_ip: extern "efiapi" fn(
        this: &Self,
        new_station_ip: Option<&IpAddress>,
        new_subnet_mask: Option<&IpAddress>,
    ) -> Status,
    set_packets: extern "efiapi" fn(
        this: &Self,
        new_dhcp_discover_valid: Option<&bool>,
        new_dhcp_ack_received: Option<&bool>,
        new_proxy_offer_received: Option<&bool>,
        new_pxe_discover_valid: Option<&bool>,
        new_pxe_reply_received: Option<&bool>,
        new_pxe_bis_reply_received: Option<&bool>,
        new_dhcp_discover: Option<&Packet>,
        new_dhcp_ack: Option<&Packet>,
        new_proxy_offer: Option<&Packet>,
        new_pxe_discover: Option<&Packet>,
        new_pxe_reply: Option<&Packet>,
        new_pxe_bis_reply: Option<&Packet>,
    ) -> Status,
    mode: *const Mode,
}

impl BaseCode {
    /// Enables the use of the PXE Base Code Protocol functions.
    pub fn start(&mut self, use_ipv6: bool) -> Result {
        (self.start)(self, use_ipv6).into()
    }

    /// Disables the use of the PXE Base Code Protocol functions.
    pub fn stop(&mut self) -> Result {
        (self.stop)(self).into()
    }

    /// Attempts to complete a DHCPv4 D.O.R.A. (discover / offer / request /
    /// acknowledge) or DHCPv6 S.A.R.R (solicit / advertise / request / reply) sequence.
    pub fn dhcp(&mut self, sort_offers: bool) -> Result {
        (self.dhcp)(self, sort_offers).into()
    }

    /// Attempts to complete the PXE Boot Server and/or boot image discovery
    /// sequence.
    pub fn discover(
        &mut self,
        ty: BootstrapType,
        layer: &mut u16,
        use_bis: bool,
        info: Option<&DiscoverInfo<[Server]>>,
    ) -> Result {
        (self.discover)(
            self,
            ty,
            layer,
            use_bis,
            info.map(|info| (info as *const DiscoverInfo<[Server]>).cast()),
        )
        .into()
    }

    /// Returns the size of a file located on a TFTP server.
    pub fn tftp_get_file_size(&mut self, server_ip: &IpAddress, filename: &CStr8) -> Result<u64> {
        let mut buffer_size = 0;

        let status = unsafe {
            (self.mtftp)(
                self,
                TftpOpcode::TftpGetFileSize,
                null_mut(),
                false,
                &mut buffer_size,
                None,
                server_ip,
                filename.as_ptr(),
                None,
                false,
            )
        };
        Result::from(status)?;

        Ok(buffer_size)
    }

    /// Reads a file located on a TFTP server.
    pub fn tftp_read_file(
        &mut self,
        server_ip: &IpAddress,
        filename: &CStr8,
        buffer: Option<&mut [u8]>,
    ) -> Result<u64> {
        let (buffer_ptr, mut buffer_size, dont_use_buffer) = if let Some(buffer) = buffer {
            let buffer_size = u64::try_from(buffer.len()).unwrap();
            ((&mut buffer[0] as *mut u8).cast(), buffer_size, false)
        } else {
            (null_mut(), 0, true)
        };

        let status = unsafe {
            (self.mtftp)(
                self,
                TftpOpcode::TftpReadFile,
                buffer_ptr,
                false,
                &mut buffer_size,
                None,
                server_ip,
                filename.as_ptr(),
                None,
                dont_use_buffer,
            )
        };
        Result::from(status)?;

        Ok(buffer_size)
    }

    /// Writes to a file located on a TFTP server.
    pub fn tftp_write_file(
        &mut self,
        server_ip: &IpAddress,
        filename: &CStr8,
        overwrite: bool,
        buffer: &[u8],
    ) -> Result {
        let buffer_ptr = (&buffer[0] as *const u8 as *mut u8).cast();
        let mut buffer_size = u64::try_from(buffer.len()).expect("buffer length should fit in u64");

        unsafe {
            (self.mtftp)(
                self,
                TftpOpcode::TftpWriteFile,
                buffer_ptr,
                overwrite,
                &mut buffer_size,
                None,
                server_ip,
                filename.as_ptr(),
                None,
                false,
            )
        }
        .into()
    }

    /// Reads a directory listing of a directory on a TFTP server.
    pub fn tftp_read_dir<'a>(
        &self,
        server_ip: &IpAddress,
        directory_name: &CStr8,
        buffer: &'a mut [u8],
    ) -> Result<impl Iterator<Item = core::result::Result<TftpFileInfo<'a>, ReadDirParseError>> + 'a>
    {
        let buffer_ptr = (&buffer[0] as *const u8 as *mut u8).cast();
        let mut buffer_size = u64::try_from(buffer.len()).expect("buffer length should fit in u64");

        let status = unsafe {
            (self.mtftp)(
                self,
                TftpOpcode::TftpReadDirectory,
                buffer_ptr,
                false,
                &mut buffer_size,
                None,
                server_ip,
                directory_name.as_ptr(),
                None,
                false,
            )
        };
        Result::from(status)?;

        let buffer_size = usize::try_from(buffer_size).expect("buffer length should fit in usize");
        let buffer = &buffer[..buffer_size];

        let mut iterator = buffer.split_inclusive(|b| *b == 0);
        let mut parse_next = move || {
            let filename = iterator.next().ok_or(ReadDirParseError)?;
            if filename == [0] {
                // This is the final entry.
                return Ok(None);
            }
            let filename = CStr8::from_bytes_with_nul(filename).unwrap();

            let information_string = iterator.next().ok_or(ReadDirParseError)?;
            let (_null_terminator, information_string) = information_string.split_last().unwrap();
            let information_string =
                core::str::from_utf8(information_string).map_err(|_| ReadDirParseError)?;

            let (size, rest) = information_string
                .split_once(' ')
                .ok_or(ReadDirParseError)?;
            let (year, rest) = rest.split_once('-').ok_or(ReadDirParseError)?;
            let (month, rest) = rest.split_once('-').ok_or(ReadDirParseError)?;
            let (day, rest) = rest.split_once(' ').ok_or(ReadDirParseError)?;
            let (hour, rest) = rest.split_once(':').ok_or(ReadDirParseError)?;
            let (minute, second) = rest.split_once(':').ok_or(ReadDirParseError)?;

            let size = size.parse().map_err(|_| ReadDirParseError)?;
            let year = year.parse().map_err(|_| ReadDirParseError)?;
            let month = month.parse().map_err(|_| ReadDirParseError)?;
            let day = day.parse().map_err(|_| ReadDirParseError)?;
            let hour = hour.parse().map_err(|_| ReadDirParseError)?;
            let minute = minute.parse().map_err(|_| ReadDirParseError)?;
            let second = second.parse().map_err(|_| ReadDirParseError)?;

            Ok(Some(TftpFileInfo {
                filename,
                size,
                year,
                month,
                day,
                hour,
                minute,
                second,
            }))
        };
        Ok(from_fn(move || parse_next().transpose()).fuse())
    }

    /// Returns the size of a file located on a MTFTP server.
    pub fn mtftp_get_file_size(
        &mut self,
        server_ip: &IpAddress,
        filename: &CStr8,
        info: &MtftpInfo,
    ) -> Result<u64> {
        let mut buffer_size = 0;

        let status = unsafe {
            (self.mtftp)(
                self,
                TftpOpcode::MtftpGetFileSize,
                null_mut(),
                false,
                &mut buffer_size,
                None,
                server_ip,
                filename.as_ptr(),
                Some(info),
                false,
            )
        };
        Result::from(status)?;

        Ok(buffer_size)
    }

    /// Reads a file located on a MTFTP server.
    pub fn mtftp_read_file(
        &mut self,
        server_ip: &IpAddress,
        filename: &CStr8,
        buffer: Option<&mut [u8]>,
        info: &MtftpInfo,
    ) -> Result<u64> {
        let (buffer_ptr, mut buffer_size, dont_use_buffer) = if let Some(buffer) = buffer {
            let buffer_size = u64::try_from(buffer.len()).unwrap();
            ((&mut buffer[0] as *mut u8).cast(), buffer_size, false)
        } else {
            (null_mut(), 0, true)
        };

        let status = unsafe {
            (self.mtftp)(
                self,
                TftpOpcode::MtftpReadFile,
                buffer_ptr,
                false,
                &mut buffer_size,
                None,
                server_ip,
                filename.as_ptr(),
                Some(info),
                dont_use_buffer,
            )
        };
        Result::from(status)?;

        Ok(buffer_size)
    }

    /// Reads a directory listing of a directory on a MTFTP server.
    pub fn mtftp_read_dir<'a>(
        &self,
        server_ip: &IpAddress,
        buffer: &'a mut [u8],
        info: &MtftpInfo,
    ) -> Result<impl Iterator<Item = core::result::Result<MtftpFileInfo<'a>, ReadDirParseError>> + 'a>
    {
        let buffer_ptr = (&buffer[0] as *const u8 as *mut u8).cast();
        let mut buffer_size = u64::try_from(buffer.len()).expect("buffer length should fit in u64");

        let status = unsafe {
            (self.mtftp)(
                self,
                TftpOpcode::MtftpReadDirectory,
                buffer_ptr,
                false,
                &mut buffer_size,
                None,
                server_ip,
                null_mut(),
                Some(info),
                false,
            )
        };
        Result::from(status)?;

        let buffer_size = usize::try_from(buffer_size).expect("buffer length should fit in usize");
        let buffer = &buffer[..buffer_size];

        let mut iterator = buffer.split_inclusive(|b| *b == 0);
        let mut parse_next = move || {
            let filename = iterator.next().ok_or(ReadDirParseError)?;
            if filename == [0] {
                // This is the final entry.
                return Ok(None);
            }
            let filename = CStr8::from_bytes_with_nul(filename).unwrap();

            let multicast_ip = iterator.next().ok_or(ReadDirParseError)?;
            let (_null_terminator, multicast_ip) = multicast_ip.split_last().unwrap();
            let multicast_ip = core::str::from_utf8(multicast_ip).map_err(|_| ReadDirParseError)?;
            let mut octets = multicast_ip.split('.');
            let mut buffer = [0; 4];
            for b in buffer.iter_mut() {
                let octet = octets.next().ok_or(ReadDirParseError)?;
                let octet = octet.parse().map_err(|_| ReadDirParseError)?;
                *b = octet;
            }
            if octets.next().is_some() {
                // The IP should have exact 4 octets, not more.
                return Err(ReadDirParseError);
            }
            let ip_address = IpAddress::new_v4(buffer);

            let information_string = iterator.next().ok_or(ReadDirParseError)?;
            let (_null_terminator, information_string) = information_string.split_last().unwrap();
            let information_string =
                core::str::from_utf8(information_string).map_err(|_| ReadDirParseError)?;

            let (size, rest) = information_string
                .split_once(' ')
                .ok_or(ReadDirParseError)?;
            let (year, rest) = rest.split_once('-').ok_or(ReadDirParseError)?;
            let (month, rest) = rest.split_once('-').ok_or(ReadDirParseError)?;
            let (day, rest) = rest.split_once(' ').ok_or(ReadDirParseError)?;
            let (hour, rest) = rest.split_once(':').ok_or(ReadDirParseError)?;
            let (minute, second) = rest.split_once(':').ok_or(ReadDirParseError)?;

            let size = size.parse().map_err(|_| ReadDirParseError)?;
            let year = year.parse().map_err(|_| ReadDirParseError)?;
            let month = month.parse().map_err(|_| ReadDirParseError)?;
            let day = day.parse().map_err(|_| ReadDirParseError)?;
            let hour = hour.parse().map_err(|_| ReadDirParseError)?;
            let minute = minute.parse().map_err(|_| ReadDirParseError)?;
            let second = second.parse().map_err(|_| ReadDirParseError)?;

            Ok(Some(MtftpFileInfo {
                filename,
                ip_address,
                size,
                year,
                month,
                day,
                hour,
                minute,
                second,
            }))
        };
        Ok(from_fn(move || parse_next().transpose()).fuse())
    }

    /// Writes a UDP packet to the network interface.
    #[allow(clippy::too_many_arguments)]
    pub fn udp_write(
        &mut self,
        op_flags: UdpOpFlags,
        dest_ip: &IpAddress,
        dest_port: u16,
        gateway_ip: Option<&IpAddress>,
        src_ip: Option<&IpAddress>,
        src_port: Option<&mut u16>,
        header: Option<&[u8]>,
        buffer: &[u8],
    ) -> Result {
        let header_size_tmp;
        let (header_size, header_ptr) = if let Some(header) = header {
            header_size_tmp = header.len();
            (Some(&header_size_tmp), (&header[0] as *const u8).cast())
        } else {
            (None, null())
        };

        unsafe {
            (self.udp_write)(
                self,
                op_flags,
                dest_ip,
                &dest_port,
                gateway_ip,
                src_ip,
                src_port,
                header_size,
                header_ptr,
                &buffer.len(),
                (&buffer[0] as *const u8).cast(),
            )
        }
        .into()
    }

    /// Reads a UDP packet from the network interface.
    #[allow(clippy::too_many_arguments)]
    pub fn udp_read(
        &mut self,
        op_flags: UdpOpFlags,
        dest_ip: Option<&mut IpAddress>,
        dest_port: Option<&mut u16>,
        src_ip: Option<&mut IpAddress>,
        src_port: Option<&mut u16>,
        header: Option<&mut [u8]>,
        buffer: &mut [u8],
    ) -> Result<usize> {
        let header_size_tmp;
        let (header_size, header_ptr) = if let Some(header) = header {
            header_size_tmp = header.len();
            (Some(&header_size_tmp), (&mut header[0] as *mut u8).cast())
        } else {
            (None, null_mut())
        };

        let mut buffer_size = buffer.len();

        let status = unsafe {
            (self.udp_read)(
                self,
                op_flags,
                dest_ip,
                dest_port,
                src_ip,
                src_port,
                header_size,
                header_ptr,
                &mut buffer_size,
                (&mut buffer[0] as *mut u8).cast(),
            )
        };
        Result::from(status)?;

        Ok(buffer_size)
    }

    /// Updates the IP receive filters of a network device and enables software
    /// filtering.
    pub fn set_ip_filter(&mut self, new_filter: &IpFilter) -> Result {
        (self.set_ip_filter)(self, new_filter).into()
    }

    /// Uses the ARP protocol to resolve a MAC address.
    pub fn arp(&mut self, ip_addr: &IpAddress, mac_addr: Option<&mut MacAddress>) -> Result {
        (self.arp)(self, ip_addr, mac_addr).into()
    }

    /// Updates the parameters that affect the operation of the PXE Base Code
    /// Protocol.
    pub fn set_parameters(
        &mut self,
        new_auto_arp: Option<bool>,
        new_send_guid: Option<bool>,
        new_ttl: Option<u8>,
        new_tos: Option<u8>,
        new_make_callback: Option<bool>,
    ) -> Result {
        (self.set_parameters)(
            self,
            new_auto_arp.as_ref(),
            new_send_guid.as_ref(),
            new_ttl.as_ref(),
            new_tos.as_ref(),
            new_make_callback.as_ref(),
        )
        .into()
    }

    /// Updates the station IP address and/or subnet mask values of a network
    /// device.
    pub fn set_station_ip(
        &mut self,
        new_station_ip: Option<&IpAddress>,
        new_subnet_mask: Option<&IpAddress>,
    ) -> Result {
        (self.set_station_ip)(self, new_station_ip, new_subnet_mask).into()
    }

    /// Updates the contents of the cached DHCP and Discover packets.
    #[allow(clippy::too_many_arguments)]
    pub fn set_packets(
        &mut self,
        new_dhcp_discover_valid: Option<bool>,
        new_dhcp_ack_received: Option<bool>,
        new_proxy_offer_received: Option<bool>,
        new_pxe_discover_valid: Option<bool>,
        new_pxe_reply_received: Option<bool>,
        new_pxe_bis_reply_received: Option<bool>,
        new_dhcp_discover: Option<&Packet>,
        new_dhcp_ack: Option<&Packet>,
        new_proxy_offer: Option<&Packet>,
        new_pxe_discover: Option<&Packet>,
        new_pxe_reply: Option<&Packet>,
        new_pxe_bis_reply: Option<&Packet>,
    ) -> Result {
        (self.set_packets)(
            self,
            new_dhcp_discover_valid.as_ref(),
            new_dhcp_ack_received.as_ref(),
            new_proxy_offer_received.as_ref(),
            new_pxe_discover_valid.as_ref(),
            new_pxe_reply_received.as_ref(),
            new_pxe_bis_reply_received.as_ref(),
            new_dhcp_discover,
            new_dhcp_ack,
            new_proxy_offer,
            new_pxe_discover,
            new_pxe_reply,
            new_pxe_bis_reply,
        )
        .into()
    }

    /// Returns a reference to the `Mode` struct.
    pub fn mode(&self) -> &Mode {
        unsafe { &*self.mode }
    }
}

/// A type of bootstrap to perform in [`BaseCode::discover`].
///
/// Corresponds to the `EFI_PXE_BASE_CODE_BOOT_` constants in the C API.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
#[allow(missing_docs)]
pub enum BootstrapType {
    Bootstrap = 0,
    MsWinntRis = 1,
    IntelLcm = 2,
    DosUndi = 3,
    NecEsmpro = 4,
    IbmWsoD = 5,
    IbmLccm = 6,
    CaUnicenterTng = 7,
    HpOpenview = 8,
    Altiris9 = 9,
    Altiris10 = 10,
    Altiris11 = 11,
    // NOT_USED_12 = 12,
    RedhatInstall = 13,
    RedhatBoot = 14,
    Rembo = 15,
    Beoboot = 16,
    //
    // Values 17 through 32767 are reserved.
    // Values 32768 through 65279 are for vendor use.
    // Values 65280 through 65534 are reserved.
    //
    PxeTest = 65535,
}

/// This struct contains optional parameters for [`BaseCode::discover`].
///
/// Corresponds to the `EFI_PXE_BASE_CODE_DISCOVER_INFO` type in the C API.
#[repr(C)]
pub struct DiscoverInfo<T: ?Sized> {
    use_m_cast: bool,
    use_b_cast: bool,
    use_u_cast: bool,
    must_use_list: bool,
    server_m_cast_ip: IpAddress,
    ip_cnt: u16,
    srv_list: T,
}

impl<const N: usize> DiscoverInfo<[Server; N]> {
    /// Create a `DiscoverInfo`.
    pub const fn new(
        use_m_cast: bool,
        use_b_cast: bool,
        use_u_cast: bool,
        must_use_list: bool,
        server_m_cast_ip: IpAddress,
        srv_list: [Server; N],
    ) -> Self {
        assert!(N <= u16::MAX as usize, "too many servers");
        let ip_cnt = N as u16;
        Self {
            use_m_cast,
            use_b_cast,
            use_u_cast,
            must_use_list,
            server_m_cast_ip,
            ip_cnt,
            srv_list,
        }
    }
}

impl<T> DiscoverInfo<T> {
    /// Returns whether discovery should use multicast.
    pub fn use_m_cast(&self) -> bool {
        self.use_m_cast
    }

    /// Returns whether discovery should use broadcast.
    pub fn use_b_cast(&self) -> bool {
        self.use_b_cast
    }

    /// Returns whether discovery should use unicast.
    pub fn use_u_cast(&self) -> bool {
        self.use_u_cast
    }

    /// Returns whether discovery should only accept boot servers in the server
    /// list (boot server verification).
    pub fn must_use_list(&self) -> bool {
        self.must_use_list
    }

    /// Returns the address used in multicast discovery.
    pub fn server_m_cast_ip(&self) -> &IpAddress {
        &self.server_m_cast_ip
    }

    /// Returns the amount of Boot Server.
    pub fn ip_cnt(&self) -> u16 {
        self.ip_cnt
    }

    /// Returns the Boot Server list used for unicast discovery or boot server
    /// verification.
    pub fn srv_list(&self) -> &T {
        &self.srv_list
    }
}

/// An entry in the Boot Server list
///
/// Corresponds to the `EFI_PXE_BASE_CODE_SRVLIST` type in the C API.
#[repr(C)]
pub struct Server {
    /// The type of Boot Server reply
    pub ty: u16,
    accept_any_response: bool,
    _reserved: u8,
    /// The IP address of the server
    ip_addr: IpAddress,
}

impl Server {
    /// Construct a `Server` for a Boot Server reply type. If `ip_addr` is not
    /// `None` only Boot Server replies with matching the IP address will be
    /// accepted.
    pub fn new(ty: u16, ip_addr: Option<IpAddress>) -> Self {
        Self {
            ty,
            accept_any_response: ip_addr.is_none(),
            _reserved: 0,
            ip_addr: ip_addr.unwrap_or(IpAddress([0; 16])),
        }
    }

    /// Returns a `None` if the any response should be accepted or the IP
    /// address of a Boot Server whose responses should be accepted.
    pub fn ip_addr(&self) -> Option<&IpAddress> {
        if self.accept_any_response {
            None
        } else {
            Some(&self.ip_addr)
        }
    }
}

/// Corresponds to the `EFI_PXE_BASE_CODE_TFTP_OPCODE` type in the C API.
#[repr(C)]
enum TftpOpcode {
    TftpGetFileSize = 1,
    TftpReadFile,
    TftpWriteFile,
    TftpReadDirectory,
    MtftpGetFileSize,
    MtftpReadFile,
    MtftpReadDirectory,
}

/// MTFTP connection parameters
///
/// Corresponds to the `EFI_PXE_BASE_CODE_MTFTP_INFO` type in the C API.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct MtftpInfo {
    /// File multicast IP address. This is the IP address to which the server
    /// will send the requested file.
    pub m_cast_ip: IpAddress,
    /// Client multicast listening port. This is the UDP port to which the
    /// server will send the requested file.
    pub c_port: u16,
    /// Server multicast listening port. This is the UDP port on which the
    /// server listens for multicast open requests and data acks.
    pub s_port: u16,
    /// The number of seconds a client should listen for an active multicast
    /// session before requesting a new multicast session.
    pub listen_timeout: u16,
    /// The number of seconds a client should wait for a packet from the server
    /// before retransmitting the previous open request or data ack packet.
    pub transmit_timeout: u16,
}

// No corresponding type in the UEFI spec, it just uses UINT16.
bitflags! {
    /// Flags for UDP read and write operations.
    #[repr(transparent)]
    pub struct UdpOpFlags: u16 {
        /// Receive a packet sent from any IP address in UDP read operations.
        const ANY_SRC_IP = 0x0001;
        /// Receive a packet sent from any UDP port in UDP read operations. If
        /// the source port is no specified in UDP write operations, the
        /// source port will be automatically selected.
        const ANY_SRC_PORT = 0x0002;
        /// Receive a packet sent to any IP address in UDP read operations.
        const ANY_DEST_IP = 0x0004;
        /// Receive a packet sent to any UDP port in UDP read operations.
        const ANY_DEST_PORT = 0x0008;
        /// The software filter is used in UDP read operations.
        const USE_FILTER = 0x0010;
        /// If required, a UDP write operation may be broken up across multiple packets.
        const MAY_FRAGMENT = 0x0020;
    }
}

/// IP receive filter settings
///
/// Corresponds to the `EFI_PXE_BASE_CODE_IP_FILTER` type in the C API.
#[repr(C)]
pub struct IpFilter {
    /// A set of filters.
    pub filters: IpFilters,
    ip_cnt: u8,
    _reserved: u16,
    ip_list: [IpAddress; 8],
}

impl IpFilter {
    /// Construct a new `IpFilter`.
    ///
    /// # Panics
    ///
    /// Panics if `ip_list` contains more than 8 entries.
    pub fn new(filters: IpFilters, ip_list: &[IpAddress]) -> Self {
        assert!(ip_list.len() <= 8);

        let ip_cnt = ip_list.len() as u8;
        let mut buffer = [IpAddress([0; 16]); 8];
        buffer[..ip_list.len()].copy_from_slice(ip_list);

        Self {
            filters,
            ip_cnt,
            _reserved: 0,
            ip_list: buffer,
        }
    }

    /// A list of IP addresses other than the Station Ip that should be
    /// enabled. Maybe be multicast or unicast.
    pub fn ip_list(&self) -> &[IpAddress] {
        &self.ip_list[..usize::from(self.ip_cnt)]
    }
}

bitflags! {
    /// IP receive filters.
    #[repr(transparent)]
    pub struct IpFilters: u8 {
        /// Enable the Station IP address.
        const STATION_IP = 0x01;
        /// Enable IPv4 broadcast addresses.
        const BROADCAST = 0x02;
        /// Enable all addresses.
        const PROMISCUOUS = 0x04;
        /// Enable all multicast addresses.
        const PROMISCUOUS_MULTICAST = 0x08;
    }
}

/// A network packet.
///
/// Corresponds to the `EFI_PXE_BASE_CODE_PACKET` type in the C API.
#[repr(C)]
pub union Packet {
    raw: [u8; 1472],
    dhcpv4: DhcpV4Packet,
    dhcpv6: DhcpV6Packet,
}

impl AsRef<[u8; 1472]> for Packet {
    fn as_ref(&self) -> &[u8; 1472] {
        unsafe { &self.raw }
    }
}

impl AsRef<DhcpV4Packet> for Packet {
    fn as_ref(&self) -> &DhcpV4Packet {
        unsafe { &self.dhcpv4 }
    }
}

impl AsRef<DhcpV6Packet> for Packet {
    fn as_ref(&self) -> &DhcpV6Packet {
        unsafe { &self.dhcpv6 }
    }
}

/// A Dhcpv4 Packet.
///
/// Corresponds to the `EFI_PXE_BASE_CODE_DHCPV4_PACKET` type in the C API.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DhcpV4Packet {
    /// Packet op code / message type.
    pub bootp_opcode: u8,
    /// Hardware address type.
    pub bootp_hw_type: u8,
    /// Hardware address length.
    pub bootp_hw_addr_len: u8,
    /// Client sets to zero, optionally used by gateways in cross-gateway booting.
    pub bootp_gate_hops: u8,
    bootp_ident: u32,
    bootp_seconds: u16,
    bootp_flags: u16,
    /// Client IP address, filled in by client in bootrequest if known.
    pub bootp_ci_addr: [u8; 4],
    /// 'your' (client) IP address; filled by server if client doesn't know its own address (`bootp_ci_addr` was 0).
    pub bootp_yi_addr: [u8; 4],
    /// Server IP address, returned in bootreply by server.
    pub bootp_si_addr: [u8; 4],
    /// Gateway IP address, used in optional cross-gateway booting.
    pub bootp_gi_addr: [u8; 4],
    /// Client hardware address, filled in by client.
    pub bootp_hw_addr: [u8; 16],
    /// Optional server host name, null terminated string.
    pub bootp_srv_name: [u8; 64],
    /// Boot file name, null terminated string, 'generic' name or null in
    /// bootrequest, fully qualified directory-path name in bootreply.
    pub bootp_boot_file: [u8; 128],
    dhcp_magik: u32,
    /// Optional vendor-specific area, e.g. could be hardware type/serial on request, or 'capability' / remote file system handle on reply.  This info may be set aside for use by a third phase bootstrap or kernel.
    pub dhcp_options: [u8; 56],
}

impl DhcpV4Packet {
    /// The expected value for [`Self::dhcp_magik`].
    pub const DHCP_MAGIK: u32 = 0x63825363;

    /// Transaction ID, a random number, used to match this boot request with the responses it generates.
    pub fn bootp_ident(&self) -> u32 {
        u32::from_be(self.bootp_ident)
    }

    /// Filled in by client, seconds elapsed since client started trying to boot.
    pub fn bootp_seconds(&self) -> u16 {
        u16::from_be(self.bootp_seconds)
    }

    /// The flags.
    pub fn bootp_flags(&self) -> DhcpV4Flags {
        DhcpV4Flags::from_bits_truncate(u16::from_be(self.bootp_flags))
    }

    /// A magic cookie, should be [`Self::DHCP_MAGIK`].
    pub fn dhcp_magik(&self) -> u32 {
        u32::from_be(self.dhcp_magik)
    }
}

bitflags! {
    /// Represents the 'flags' field for a [`DhcpV4Packet`].
    pub struct DhcpV4Flags: u16 {
        /// Should be set when the client cannot receive unicast IP datagrams
        /// until its protocol software has been configured with an IP address.
        const BROADCAST = 1;
    }
}

/// A Dhcpv6 Packet.
///
/// Corresponds to the `EFI_PXE_BASE_CODE_DHCPV6_PACKET` type in the C API.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DhcpV6Packet {
    /// The message type.
    pub message_type: u8,
    transaction_id: [u8; 3],
    /// A byte array containing dhcp options.
    pub dhcp_options: [u8; 1024],
}

impl DhcpV6Packet {
    /// The transaction id.
    pub fn transaction_id(&self) -> u32 {
        u32::from(self.transaction_id[0]) << 16
            | u32::from(self.transaction_id[1]) << 8
            | u32::from(self.transaction_id[2])
    }
}

/// The data values in this structure are read-only and are updated by the
/// [`BaseCode`].
///
/// Corresponds to the `EFI_PXE_BASE_CODE_MODE` type in the C API.
#[repr(C)]
pub struct Mode {
    /// `true` if this device has been started by calling [`BaseCode::start`].
    /// This field is set to `true` by [`BaseCode::start`] and to `false` by
    /// the [`BaseCode::stop`] function.
    pub started: bool,
    /// `true` if the UNDI protocol supports IPv6
    pub ipv6_available: bool,
    /// `true` if this PXE Base Code Protocol implementation supports IPv6.
    pub ipv6_supported: bool,
    /// `true` if this device is currently using IPv6. This field is set by
    /// [`BaseCode::start`].
    pub using_ipv6: bool,
    /// `true` if this PXE Base Code implementation supports Boot Integrity
    /// Services (BIS). This field is set by [`BaseCode::start`].
    pub bis_supported: bool,
    /// `true` if this device and the platform support Boot Integrity Services
    /// (BIS). This field is set by [`BaseCode::start`].
    pub bis_detected: bool,
    /// `true` for automatic ARP packet generation, `false` otherwise. This
    /// field is initialized to `true` by [`BaseCode::start`] and can be
    /// modified with [`BaseCode::set_parameters`].
    pub auto_arp: bool,
    /// This field is used to change the Client Hardware Address (chaddr) field
    /// in the DHCP and Discovery packets. Set to `true` to send the SystemGuid
    /// (if one is available). Set to `false` to send the client NIC MAC
    /// address. This field is initialized to `false` by [`BaseCode::start`]
    /// and can be modified with [`BaseCode::set_parameters`].
    pub send_guid: bool,
    /// This field is initialized to `false` by [`BaseCode::start`] and set to
    /// `true` when [`BaseCode::dhcp`] completes successfully. When `true`,
    /// [`Self::dhcp_discover`] is valid. This field can also be changed by
    /// [`BaseCode::set_packets`].
    pub dhcp_discover_valid: bool,
    /// This field is initialized to `false` by [`BaseCode::start`] and set to
    /// `true` when [`BaseCode::dhcp`] completes successfully. When `true`,
    /// [`Self::dhcp_ack`] is valid. This field can also be changed by
    /// [`BaseCode::set_packets`].
    pub dhcp_ack_received: bool,
    /// This field is initialized to `false` by [`BaseCode::start`] and set to
    /// `true` when [`BaseCode::dhcp`] completes successfully and a proxy DHCP
    /// offer packet was received. When `true`, [`Self::proxy_offer`] is valid.
    /// This field can also be changed by [`BaseCode::set_packets`].
    pub proxy_offer_received: bool,
    /// When `true`, [`Self::pxe_discover`] is valid. This field is set to
    /// `false` by [`BaseCode::start`] and [`BaseCode::dhcp`], and can be set
    /// to `true` or `false` by [`BaseCode::discover`] and
    /// [`BaseCode::set_packets`].
    pub pxe_discover_valid: bool,
    /// When `true`, [`Self::pxe_reply`] is valid. This field is set to `false`
    /// by [`BaseCode::start`] and [`BaseCode::dhcp`], and can be set to `true`
    /// or `false` by [`BaseCode::discover`] and [`BaseCode::set_packets`].
    pub pxe_reply_received: bool,
    /// When `true`, [`Self::pxe_bis_reply`] is valid. This field is set to
    /// `false` by [`BaseCode::start`] and [`BaseCode::dhcp`], and can be set
    /// to `true` or `false` by the [`BaseCode::discover`] and
    /// [`BaseCode::set_packets`].
    pub pxe_bis_reply_received: bool,
    /// Indicates whether [`Self::icmp_error`] has been updated. This field is
    /// reset to `false` by [`BaseCode::start`], [`BaseCode::dhcp`],
    /// [`BaseCode::discover`],[`BaseCode::udp_read`], [`BaseCode::udp_write`],
    /// [`BaseCode::arp`] and any of the TFTP/MTFTP operations. If an ICMP
    /// error is received, this field will be set to `true` after
    /// [`Self::icmp_error`] is updated.
    pub icmp_error_received: bool,
    /// Indicates whether [`Self::tftp_error`] has been updated. This field is
    /// reset to `false` by [`BaseCode::start`] and any of the TFTP/MTFTP
    /// operations. If a TFTP error is received, this field will be set to
    /// `true` after [`Self::tftp_error`] is updated.
    pub tftp_error_received: bool,
    /// When `false`, callbacks will not be made. When `true`, make callbacks
    /// to the PXE Base Code Callback Protocol. This field is reset to `false`
    /// by [`BaseCode::start`] if the PXE Base Code Callback Protocol is not
    /// available. It is reset to `true` by [`BaseCode::start`] if the PXE Base
    /// Code Callback Protocol is available.
    pub make_callbacks: bool,
    /// The "time to live" field of the IP header. This field is initialized to
    /// `16` by [`BaseCode::start`] and can be modified by
    /// [`BaseCode::set_parameters`].
    pub ttl: u8,
    /// The type of service field of the IP header. This field is initialized
    /// to `0` by [`BaseCode::start`], and can be modified with
    /// [`BaseCode::set_parameters`].
    pub tos: u8,
    /// The device’s current IP address. This field is initialized to a zero
    /// address by Start(). This field is set when [`BaseCode::dhcp`] completes
    /// successfully. This field can also be set by
    /// [`BaseCode::set_station_ip`]. This field must be set to a valid IP
    /// address by either [`BaseCode::dhcp`] or [`BaseCode::set_station_ip`]
    /// before [`BaseCode::discover`], [`BaseCode::udp_read`],
    /// [`BaseCode::udp_write`], [`BaseCode::arp`] and any of the TFTP/MTFTP
    /// operations are called.
    pub station_ip: IpAddress,
    /// The device's current subnet mask. This field is initialized to a zero
    /// address by [`BaseCode::start`]. This field is set when
    /// [`BaseCode::dhcp`] completes successfully. This field can also be set
    /// by [`BaseCode::set_station_ip`]. This field must be set to a valid
    /// subnet mask by either [`BaseCode::dhcp`] or
    /// [`BaseCode::set_station_ip`] before [`BaseCode::discover`],
    /// [`BaseCode::udp_read`], [`BaseCode::udp_write`],
    /// [`BaseCode::arp`] or any of the TFTP/MTFTP operations are called.
    pub subnet_mask: IpAddress,
    /// Cached DHCP Discover packet. This field is zero-filled by the
    /// [`BaseCode::start`] function, and is set when [`BaseCode::dhcp`]
    /// completes successfully. The contents of this field can replaced by
    /// [`BaseCode::set_packets`].
    pub dhcp_discover: Packet,
    /// Cached DHCP Ack packet. This field is zero-filled by
    /// [`BaseCode::start`], and is set when [`BaseCode::dhcp`] completes
    /// successfully. The contents of this field can be replaced by
    /// [`BaseCode::set_packets`].
    pub dhcp_ack: Packet,
    /// Cached Proxy Offer packet. This field is zero-filled by
    /// [`BaseCode::start`], and is set when [`BaseCode::dhcp`] completes
    /// successfully. The contents of this field can be replaced by
    /// [`BaseCode::set_packets`].
    pub proxy_offer: Packet,
    /// Cached PXE Discover packet. This field is zero-filled by
    /// [`BaseCode::start`], and is set when [`BaseCode::discover`] completes
    /// successfully. The contents of this field can be replaced by
    /// [`BaseCode::set_packets`].
    pub pxe_discover: Packet,
    /// Cached PXE Reply packet. This field is zero-filled by
    /// [`BaseCode::start`], and is set when [`BaseCode::discover`] completes
    /// successfully. The contents of this field can be replaced by the
    /// [`BaseCode::set_packets`] function.
    pub pxe_reply: Packet,
    /// Cached PXE BIS Reply packet. This field is zero-filled by
    /// [`BaseCode::start`], and is set when [`BaseCode::discover`] completes
    /// successfully. This field can be replaced by [`BaseCode::set_packets`].
    pub pxe_bis_reply: Packet,
    /// The current IP receive filter settings. The receive filter is disabled
    /// and the number of IP receive filters is set to zero by
    /// [`BaseCode::start`], and is set by [`BaseCode::set_ip_filter`].
    pub ip_filter: IpFilter,
    /// The number of valid entries in the ARP cache. This field is reset to
    /// zero by [`BaseCode::start`].
    pub arp_cache_entries: u32,
    /// Array of cached ARP entries.
    pub arp_cache: [ArpEntry; 8],
    /// The number of valid entries in the current route table. This field is
    /// reset to zero by [`BaseCode::start`].
    pub route_table_entries: u32,
    /// Array of route table entries.
    pub route_table: [RouteEntry; 8],
    /// ICMP error packet. This field is updated when an ICMP error is received
    /// and is undefined until the first ICMP error is received. This field is
    /// zero-filled by [`BaseCode::start`].
    pub icmp_error: IcmpError,
    /// TFTP error packet. This field is updated when a TFTP error is received
    /// and is undefined until the first TFTP error is received. This field is
    /// zero-filled by the [`BaseCode::start`] function.
    pub tftp_error: TftpError,
}

/// An entry for the ARP cache found in [`Mode::arp_cache`]
///
/// Corresponds to the `EFI_PXE_BASE_CODE_ARP_ENTRY` type in the C API.
#[repr(C)]
pub struct ArpEntry {
    /// The IP address.
    pub ip_addr: IpAddress,
    /// The mac address of the device that is addressed by [`Self::ip_addr`].
    pub mac_addr: MacAddress,
}

/// An entry for the route table found in [`Mode::route_table`]
///
/// Corresponds to the `EFI_PXE_BASE_CODE_ROUTE_ENTRY` type in the C API.
#[repr(C)]
#[allow(missing_docs)]
pub struct RouteEntry {
    pub ip_addr: IpAddress,
    pub subnet_mask: IpAddress,
    pub gw_addr: IpAddress,
}

/// An ICMP error packet.
///
/// Corresponds to the `EFI_PXE_BASE_CODE_ICMP_ERROR` type in the C API.
#[repr(C)]
#[allow(missing_docs)]
pub struct IcmpError {
    pub ty: u8,
    pub code: u8,
    pub checksum: u16,
    pub u: IcmpErrorUnion,
    pub data: [u8; 494],
}

/// Corresponds to the anonymous union inside
/// `EFI_PXE_BASE_CODE_ICMP_ERROR` in the C API.
#[repr(C)]
#[allow(missing_docs)]
pub union IcmpErrorUnion {
    pub reserved: u32,
    pub mtu: u32,
    pub pointer: u32,
    pub echo: IcmpErrorEcho,
}

/// Corresponds to the `Echo` field in the anonymous union inside
/// `EFI_PXE_BASE_CODE_ICMP_ERROR` in the C API.
#[repr(C)]
#[derive(Clone, Copy)]
#[allow(missing_docs)]
pub struct IcmpErrorEcho {
    pub identifier: u16,
    pub sequence: u16,
}

/// A TFTP error packet.
///
/// Corresponds to the `EFI_PXE_BASE_CODE_TFTP_ERROR` type in the C API.
#[repr(C)]
#[allow(missing_docs)]
pub struct TftpError {
    pub error_code: u8,
    pub error_string: [u8; 127],
}

/// Returned by [`BaseCode::tftp_read_dir`].
#[allow(missing_docs)]
pub struct TftpFileInfo<'a> {
    pub filename: &'a CStr8,
    pub size: u64,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: f32,
}

/// Returned by [`BaseCode::mtftp_read_dir`].
#[allow(missing_docs)]
pub struct MtftpFileInfo<'a> {
    pub filename: &'a CStr8,
    pub ip_address: IpAddress,
    pub size: u64,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: f32,
}

/// Returned if a server sends a malformed response in
/// [`BaseCode::tftp_read_dir`] or [`BaseCode::mtftp_read_dir`].
#[derive(Clone, Copy, Debug)]
pub struct ReadDirParseError;
