// SPDX-License-Identifier: MIT OR Apache-2.0

//! PXE Base Code protocol.

use crate::polyfill::maybe_uninit_slice_as_mut_ptr;
use crate::proto::unsafe_protocol;
use crate::util::{ptr_write_unaligned_and_add, usize_from_u32};
use crate::{CStr8, Result, Status, StatusExt};
use core::fmt::{self, Debug, Display, Formatter};
use core::iter::from_fn;
use core::mem::MaybeUninit;
use core::net::{IpAddr, Ipv4Addr};
use core::ptr::{self, null, null_mut};
use ptr_meta::Pointee;
use uefi::proto::network::EfiMacAddr;
use uefi_raw::protocol::network::pxe::{
    PxeBaseCodeDiscoverInfo, PxeBaseCodeMode, PxeBaseCodeProtocol, PxeBaseCodeTftpOpcode,
};
use uefi_raw::{Boolean, Char8, IpAddress as EfiIpAddr};

pub use uefi_raw::protocol::network::pxe::{
    PxeBaseCodeArpEntry as ArpEntry, PxeBaseCodeBootType as BootstrapType,
    PxeBaseCodeDhcpV4Flags as DhcpV4Flags, PxeBaseCodeDhcpV4Packet as DhcpV4Packet,
    PxeBaseCodeDhcpV6Packet as DhcpV6Packet, PxeBaseCodeIcmpError as IcmpError,
    PxeBaseCodeIcmpErrorEcho as IcmpErrorEcho, PxeBaseCodeIcmpErrorUnion as IcmpErrorUnion,
    PxeBaseCodeIpFilter as IpFilter, PxeBaseCodeIpFilterFlags as IpFilters,
    PxeBaseCodeMtftpInfo as MtftpInfo, PxeBaseCodePacket as Packet,
    PxeBaseCodeRouteEntry as RouteEntry, PxeBaseCodeSrvlist as Server,
    PxeBaseCodeTftpError as TftpError, PxeBaseCodeUdpOpFlags as UdpOpFlags,
};

/// PXE Base Code [`Protocol`].
///
/// [`Protocol`]: uefi::proto::Protocol
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(PxeBaseCodeProtocol::GUID)]
pub struct BaseCode(PxeBaseCodeProtocol);

impl BaseCode {
    /// Enables the use of the PXE Base Code Protocol functions.
    pub fn start(&mut self, use_ipv6: bool) -> Result {
        // SAFETY: The memory is valid.
        unsafe { (self.0.start)(&mut self.0, use_ipv6.into()) }.to_result()
    }

    /// Disables the use of the PXE Base Code Protocol functions.
    pub fn stop(&mut self) -> Result {
        // SAFETY: The memory is valid.
        unsafe { (self.0.stop)(&mut self.0) }.to_result()
    }

    /// Attempts to complete a DHCPv4 D.O.R.A. (discover / offer / request /
    /// acknowledge) or DHCPv6 S.A.R.R (solicit / advertise / request / reply) sequence.
    pub fn dhcp(&mut self, sort_offers: bool) -> Result {
        // SAFETY: The memory is valid.
        unsafe { (self.0.dhcp)(&mut self.0, sort_offers.into()) }.to_result()
    }

    /// Attempts to complete the PXE Boot Server and/or boot image discovery
    /// sequence.
    pub fn discover(
        &mut self,
        ty: BootstrapType,
        layer: &mut u16,
        use_bis: bool,
        info: Option<&DiscoverInfo>,
    ) -> Result {
        let info: *const PxeBaseCodeDiscoverInfo = info
            .map(|info| ptr::from_ref(info).cast())
            .unwrap_or(null());

        // SAFETY: The memory is valid.
        unsafe { (self.0.discover)(&mut self.0, ty, layer, use_bis.into(), info) }.to_result()
    }

    /// Returns the size of a file located on a TFTP server.
    pub fn tftp_get_file_size(&mut self, server_ip: &IpAddr, filename: &CStr8) -> Result<u64> {
        let mut buffer_size = 0;

        let server_ip = EfiIpAddr::from(*server_ip);
        // SAFETY: The memory is valid.
        let status = unsafe {
            (self.0.mtftp)(
                &mut self.0,
                PxeBaseCodeTftpOpcode::TFTP_GET_FILE_SIZE,
                null_mut(),
                Boolean::FALSE,
                &mut buffer_size,
                null(),
                &server_ip,
                cstr8_to_ptr(filename),
                null(),
                Boolean::FALSE,
            )
        };
        status.to_result_with_val(|| buffer_size)
    }

    /// Reads a file located on a TFTP server.
    pub fn tftp_read_file(
        &mut self,
        server_ip: &IpAddr,
        filename: &CStr8,
        buffer: Option<&mut [u8]>,
    ) -> Result<u64> {
        let (buffer_ptr, mut buffer_size, dont_use_buffer) = if let Some(buffer) = buffer {
            let buffer_size = u64::try_from(buffer.len()).unwrap();
            (buffer.as_mut_ptr().cast(), buffer_size, Boolean::FALSE)
        } else {
            (null_mut(), 0, Boolean::TRUE)
        };

        let server_ip = EfiIpAddr::from(*server_ip);
        // SAFETY: The memory is valid.
        let status = unsafe {
            (self.0.mtftp)(
                &mut self.0,
                PxeBaseCodeTftpOpcode::TFTP_READ_FILE,
                buffer_ptr,
                Boolean::FALSE,
                &mut buffer_size,
                null(),
                &server_ip,
                cstr8_to_ptr(filename),
                null(),
                dont_use_buffer,
            )
        };
        status.to_result_with_val(|| buffer_size)
    }

    /// Writes to a file located on a TFTP server.
    pub fn tftp_write_file(
        &mut self,
        server_ip: &IpAddr,
        filename: &CStr8,
        overwrite: bool,
        buffer: &[u8],
    ) -> Result {
        let buffer_ptr = buffer.as_ptr().cast_mut().cast();
        let mut buffer_size = u64::try_from(buffer.len()).expect("buffer length should fit in u64");

        let server_ip = EfiIpAddr::from(*server_ip);
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.mtftp)(
                &mut self.0,
                PxeBaseCodeTftpOpcode::TFTP_WRITE_FILE,
                buffer_ptr,
                overwrite.into(),
                &mut buffer_size,
                null(),
                &server_ip,
                cstr8_to_ptr(filename),
                null(),
                Boolean::FALSE,
            )
        }
        .to_result()
    }

    /// Reads a directory listing of a directory on a TFTP server.
    pub fn tftp_read_dir<'a>(
        &mut self,
        server_ip: &IpAddr,
        directory_name: &CStr8,
        buffer: &'a mut [u8],
    ) -> Result<impl Iterator<Item = core::result::Result<TftpFileInfo<'a>, ReadDirParseError>> + 'a>
    {
        let buffer_ptr = buffer.as_mut_ptr().cast();
        let mut buffer_size = u64::try_from(buffer.len()).expect("buffer length should fit in u64");

        let server_ip = EfiIpAddr::from(*server_ip);
        // SAFETY: The memory is valid.
        let status = unsafe {
            (self.0.mtftp)(
                &mut self.0,
                PxeBaseCodeTftpOpcode::TFTP_READ_DIRECTORY,
                buffer_ptr,
                Boolean::FALSE,
                &mut buffer_size,
                null(),
                &server_ip,
                cstr8_to_ptr(directory_name),
                null(),
                Boolean::FALSE,
            )
        };
        status.to_result()?;

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
        server_ip: &IpAddr,
        filename: &CStr8,
        info: &MtftpInfo,
    ) -> Result<u64> {
        let mut buffer_size = 0;

        let server_ip = EfiIpAddr::from(*server_ip);
        // SAFETY: The memory is valid.
        let status = unsafe {
            (self.0.mtftp)(
                &mut self.0,
                PxeBaseCodeTftpOpcode::MTFTP_GET_FILE_SIZE,
                null_mut(),
                Boolean::FALSE,
                &mut buffer_size,
                null(),
                &server_ip,
                cstr8_to_ptr(filename),
                info,
                Boolean::FALSE,
            )
        };
        status.to_result_with_val(|| buffer_size)
    }

    /// Reads a file located on a MTFTP server.
    pub fn mtftp_read_file(
        &mut self,
        server_ip: &IpAddr,
        filename: &CStr8,
        buffer: Option<&mut [u8]>,
        info: &MtftpInfo,
    ) -> Result<u64> {
        let (buffer_ptr, mut buffer_size, dont_use_buffer) = if let Some(buffer) = buffer {
            let buffer_size = u64::try_from(buffer.len()).unwrap();
            (buffer.as_mut_ptr().cast(), buffer_size, Boolean::FALSE)
        } else {
            (null_mut(), 0, Boolean::TRUE)
        };

        let server_ip = EfiIpAddr::from(*server_ip);
        // SAFETY: The memory is valid.
        let status = unsafe {
            (self.0.mtftp)(
                &mut self.0,
                PxeBaseCodeTftpOpcode::MTFTP_READ_FILE,
                buffer_ptr,
                Boolean::FALSE,
                &mut buffer_size,
                null(),
                &server_ip,
                cstr8_to_ptr(filename),
                info,
                dont_use_buffer,
            )
        };
        status.to_result_with_val(|| buffer_size)
    }

    /// Reads a directory listing of a directory on a MTFTP server.
    pub fn mtftp_read_dir<'a>(
        &mut self,
        server_ip: &IpAddr,
        buffer: &'a mut [u8],
        info: &MtftpInfo,
    ) -> Result<impl Iterator<Item = core::result::Result<MtftpFileInfo<'a>, ReadDirParseError>> + 'a>
    {
        let buffer_ptr = buffer.as_mut_ptr().cast();
        let mut buffer_size = u64::try_from(buffer.len()).expect("buffer length should fit in u64");

        let server_ip = EfiIpAddr::from(*server_ip);
        // SAFETY: The memory is valid.
        let status = unsafe {
            (self.0.mtftp)(
                &mut self.0,
                PxeBaseCodeTftpOpcode::MTFTP_READ_DIRECTORY,
                buffer_ptr,
                Boolean::FALSE,
                &mut buffer_size,
                null(),
                &server_ip,
                null_mut(),
                info,
                Boolean::FALSE,
            )
        };
        status.to_result()?;

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
            let ip_address = IpAddr::V4(Ipv4Addr::from(buffer));

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
    #[expect(clippy::too_many_arguments)]
    pub fn udp_write(
        &mut self,
        op_flags: UdpOpFlags,
        dest_ip: &IpAddr,
        dest_port: u16,
        gateway_ip: Option<&IpAddr>,
        src_ip: Option<&IpAddr>,
        src_port: Option<&mut u16>,
        header: Option<&[u8]>,
        buffer: &[u8],
    ) -> Result {
        let header_size_tmp;
        let (header_size, header_ptr) = if let Some(header) = header {
            header_size_tmp = header.len();
            (Some(&header_size_tmp), header.as_ptr().cast())
        } else {
            (None, null())
        };

        let dest_ip = EfiIpAddr::from(*dest_ip);
        let gateway_ip = gateway_ip.map(|ip| EfiIpAddr::from(*ip));
        let src_ip = src_ip.map(|ip| EfiIpAddr::from(*ip));
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.udp_write)(
                &mut self.0,
                op_flags,
                &dest_ip,
                &dest_port,
                opt_ip_addr_to_ptr(gateway_ip.as_ref()),
                opt_ip_addr_to_ptr(src_ip.as_ref()),
                opt_mut_to_ptr(src_port),
                opt_ref_to_ptr(header_size),
                header_ptr,
                &buffer.len(),
                buffer.as_ptr().cast(),
            )
        }
        .to_result()
    }

    /// Reads a UDP packet from the network interface.
    ///
    /// # Arguments
    /// - `op_flags`: [`UdpOpFlags`] specifying the behavior of the function
    /// - `dest_ip`: The behavior depends on whether
    ///   [`UdpOpFlags::ANY_DEST_IP`] is set. If it not set, packets must match
    ///   the specified IP. Otherwise, the corresponding IP is written into the
    ///   provided buffer.
    /// - `dest_port`: The behavior depends on whether
    ///   [`UdpOpFlags::ANY_DEST_PORT`] is set. If it not set, packets must match
    ///   the specified port. Otherwise, the corresponding port is written into
    ///   the provided buffer.
    /// - `src_ip`: The behavior depends on whether
    ///   [`UdpOpFlags::ANY_SRC_IP`] is set. If it not set, packets must match
    ///   the specified IP. Otherwise, the corresponding IP is written into the
    ///   provided buffer.
    /// - `src_port`: The behavior depends on whether
    ///   [`UdpOpFlags::ANY_SRC_PORT`] is set. If it not set, packets must match
    ///   the specified port. Otherwise, the corresponding port is written into
    ///   the provided buffer.
    /// - `header`: Optional header of the data inside `buffer`.
    /// - `buffer`: Buffer for the UDP packet's content.
    #[expect(clippy::too_many_arguments)]
    pub fn udp_read(
        &mut self,
        op_flags: UdpOpFlags,
        mut dest_ip: Option<&mut IpAddr>,
        dest_port: Option<&mut u16>,
        mut src_ip: Option<&mut IpAddr>,
        src_port: Option<&mut u16>,
        header: Option<&mut [u8]>,
        buffer: &mut [u8],
    ) -> Result<usize> {
        let header_size_tmp;
        let (header_size, header_ptr) = if let Some(header) = header {
            header_size_tmp = header.len();
            (ptr::from_ref(&header_size_tmp), header.as_mut_ptr().cast())
        } else {
            (null(), null_mut())
        };

        let mut buffer_size = buffer.len();

        let mut dest_ip_efi = dest_ip.as_ref().map(|ip| EfiIpAddr::from(**ip));
        let mut src_ip_efi = src_ip.as_ref().map(|ip| EfiIpAddr::from(**ip));

        // SAFETY: The memory is valid.
        let status = unsafe {
            (self.0.udp_read)(
                &mut self.0,
                op_flags,
                opt_ip_addr_to_ptr_mut(dest_ip_efi.as_mut()),
                opt_mut_to_ptr(dest_port),
                opt_ip_addr_to_ptr_mut(src_ip_efi.as_mut()),
                opt_mut_to_ptr(src_port),
                header_size,
                header_ptr,
                &mut buffer_size,
                buffer.as_mut_ptr().cast(),
            )
        };

        // Translate IP types back into the higher-level types.
        let fn_replace_ip = |core_ip: &mut Option<&mut IpAddr>, efi_ip: Option<EfiIpAddr>| {
            if let Some(core_ip_location) = core_ip
                && let Some(efi_ip) = efi_ip
            {
                // SAFETY: We trust that `using_ipv6()` tells the truth.
                let core_ip = unsafe { efi_ip.into_core_addr(self.mode().using_ipv6()) };
                **core_ip_location = core_ip;
            }
        };

        fn_replace_ip(&mut dest_ip, dest_ip_efi);
        fn_replace_ip(&mut src_ip, src_ip_efi);

        status.to_result_with_val(|| buffer_size)
    }

    /// Updates the IP receive filters of a network device and enables software
    /// filtering.
    pub fn set_ip_filter(&mut self, new_filter: &IpFilter) -> Result {
        // SAFETY: The memory is valid.
        unsafe { (self.0.set_ip_filter)(&mut self.0, new_filter) }.to_result()
    }

    /// Uses the ARP protocol to resolve a MAC address.
    pub fn arp(&mut self, ip_addr: &IpAddr, mac_addr: Option<&mut EfiMacAddr>) -> Result {
        let ip_addr = EfiIpAddr::from(*ip_addr);
        // SAFETY: The memory is valid.
        unsafe { (self.0.arp)(&mut self.0, &ip_addr, opt_mut_to_ptr(mac_addr)) }.to_result()
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
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.set_parameters)(
                &mut self.0,
                opt_bool_to_ptr(&new_auto_arp),
                opt_bool_to_ptr(&new_send_guid),
                opt_ref_to_ptr(new_ttl.as_ref()),
                opt_ref_to_ptr(new_tos.as_ref()),
                opt_bool_to_ptr(&new_make_callback),
            )
        }
        .to_result()
    }

    /// Updates the station IP address and/or subnet mask values of a network
    /// device.
    pub fn set_station_ip(
        &mut self,
        new_station_ip: Option<&IpAddr>,
        new_subnet_mask: Option<&IpAddr>,
    ) -> Result {
        let new_station_ip = new_station_ip.map(|ip| EfiIpAddr::from(*ip));
        let new_subnet_mask = new_subnet_mask.map(|mask| EfiIpAddr::from(*mask));
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.set_station_ip)(
                &mut self.0,
                opt_ip_addr_to_ptr(new_station_ip.as_ref()),
                opt_ip_addr_to_ptr(new_subnet_mask.as_ref()),
            )
        }
        .to_result()
    }

    /// Updates the contents of the cached DHCP and Discover packets.
    #[expect(clippy::too_many_arguments)]
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
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.set_packets)(
                &mut self.0,
                opt_bool_to_ptr(&new_dhcp_discover_valid),
                opt_bool_to_ptr(&new_dhcp_ack_received),
                opt_bool_to_ptr(&new_proxy_offer_received),
                opt_bool_to_ptr(&new_pxe_discover_valid),
                opt_bool_to_ptr(&new_pxe_reply_received),
                opt_bool_to_ptr(&new_pxe_bis_reply_received),
                opt_ref_to_ptr(new_dhcp_discover),
                opt_ref_to_ptr(new_dhcp_ack),
                opt_ref_to_ptr(new_proxy_offer),
                opt_ref_to_ptr(new_pxe_discover),
                opt_ref_to_ptr(new_pxe_reply),
                opt_ref_to_ptr(new_pxe_bis_reply),
            )
        }
        .to_result()
    }

    /// Returns a reference to the `Mode` struct.
    #[must_use]
    pub const fn mode(&self) -> &Mode {
        // SAFETY: The memory is valid.
        unsafe { &*(self.0.mode.cast()) }
    }
}

/// Convert a `&CStr8` to a `*const uefi_raw::Char8`.
const fn cstr8_to_ptr(s: &CStr8) -> *const Char8 {
    s.as_ptr().cast()
}

/// Convert an `&Option<bool>` to a `*const Boolean`.
///
/// This is always a valid conversion; `bool` is an 8-bit `0` or `1`.
fn opt_bool_to_ptr(arg: &Option<bool>) -> *const Boolean {
    arg.as_ref()
        .map(|arg| ptr::from_ref(arg).cast::<Boolean>())
        .unwrap_or_else(null)
}

/// Convert an `Option<&EfiIpAddr>` to a `*const uefi_raw::EfiIpAddr`.
fn opt_ip_addr_to_ptr(arg: Option<&EfiIpAddr>) -> *const EfiIpAddr {
    arg.map(|arg| &raw const *arg).unwrap_or_else(null)
}

/// Convert an `Option<&mut EfiIpAddr>` to a `*mut uefi_raw::EfiIpAddr`.
fn opt_ip_addr_to_ptr_mut(arg: Option<&mut EfiIpAddr>) -> *mut EfiIpAddr {
    arg.map(|arg| &raw mut *arg).unwrap_or_else(null_mut)
}

/// Convert an `Option<&T>` to a `*const T`.
fn opt_ref_to_ptr<T>(opt: Option<&T>) -> *const T {
    opt.map(ptr::from_ref).unwrap_or(null())
}

/// Convert an `Option<&mut T>` to a `*mut T`.
fn opt_mut_to_ptr<T>(opt: Option<&mut T>) -> *mut T {
    opt.map(ptr::from_mut).unwrap_or(null_mut())
}

opaque_type! {
    /// Opaque type that should be used to represent a pointer to a [`DiscoverInfo`] in
    /// foreign function interfaces. This type produces a thin pointer, unlike
    /// [`DiscoverInfo`].
    pub struct FfiDiscoverInfo;
}

/// This struct contains optional parameters for [`BaseCode::discover`].
///
/// Corresponds to the `EFI_PXE_BASE_CODE_DISCOVER_INFO` type in the C API.
#[repr(C)]
#[derive(Debug, Pointee)]
pub struct DiscoverInfo {
    use_m_cast: bool,
    use_b_cast: bool,
    use_u_cast: bool,
    must_use_list: bool,
    server_m_cast_ip: EfiIpAddr,
    ip_cnt: u16,
    srv_list: [Server],
}

impl DiscoverInfo {
    /// Create a `DiscoverInfo`.
    pub fn new_in_buffer<'buf>(
        buffer: &'buf mut [MaybeUninit<u8>],
        use_m_cast: bool,
        use_b_cast: bool,
        use_u_cast: bool,
        must_use_list: bool,
        server_m_cast_ip: EfiIpAddr,
        srv_list: &[Server],
    ) -> Result<&'buf mut Self> {
        let server_count = srv_list.len();
        assert!(server_count <= u16::MAX as usize, "too many servers");

        let required_size = size_of::<bool>() * 4
            + size_of::<EfiIpAddr>()
            + size_of::<u16>()
            + size_of_val(srv_list);

        if buffer.len() < required_size {
            return Err(Status::BUFFER_TOO_SMALL.into());
        }

        let mut ptr: *mut u8 = maybe_uninit_slice_as_mut_ptr(buffer);
        // SAFETY: The memory is valid.
        unsafe {
            ptr_write_unaligned_and_add(&mut ptr, use_m_cast);
            ptr_write_unaligned_and_add(&mut ptr, use_b_cast);
            ptr_write_unaligned_and_add(&mut ptr, use_u_cast);
            ptr_write_unaligned_and_add(&mut ptr, must_use_list);
            ptr_write_unaligned_and_add(&mut ptr, server_m_cast_ip);
            ptr_write_unaligned_and_add(&mut ptr, server_count as u16);

            ptr = ptr.add(2); // Align server list (4-byte alignment).
            core::ptr::copy(srv_list.as_ptr(), ptr.cast(), server_count);

            let ptr: *mut Self =
                ptr_meta::from_raw_parts_mut(buffer.as_mut_ptr().cast(), server_count);
            Ok(&mut *ptr)
        }
    }
}

impl DiscoverInfo {
    /// Returns whether discovery should use multicast.
    #[must_use]
    pub const fn use_m_cast(&self) -> bool {
        self.use_m_cast
    }

    /// Returns whether discovery should use broadcast.
    #[must_use]
    pub const fn use_b_cast(&self) -> bool {
        self.use_b_cast
    }

    /// Returns whether discovery should use unicast.
    #[must_use]
    pub const fn use_u_cast(&self) -> bool {
        self.use_u_cast
    }

    /// Returns whether discovery should only accept boot servers in the server
    /// list (boot server verification).
    #[must_use]
    pub const fn must_use_list(&self) -> bool {
        self.must_use_list
    }

    /// Returns the address used in multicast discovery.
    #[must_use]
    pub const fn server_m_cast_ip(&self) -> &EfiIpAddr {
        &self.server_m_cast_ip
    }

    /// Returns the amount of Boot Server.
    #[must_use]
    pub const fn ip_cnt(&self) -> u16 {
        self.ip_cnt
    }

    /// Returns the Boot Server list used for unicast discovery or boot server
    /// verification.
    #[must_use]
    pub const fn srv_list(&self) -> &[Server] {
        &self.srv_list
    }
}

/// The data values in this structure are read-only and are updated by the
/// [`BaseCode`].
///
/// Corresponds to the `EFI_PXE_BASE_CODE_MODE` type in the C API.
#[repr(transparent)]
#[derive(Debug)]
pub struct Mode(PxeBaseCodeMode);

impl Mode {
    /// `true` if this device has been started by calling [`BaseCode::start`].
    /// This field is set to `true` by [`BaseCode::start`] and to `false` by
    /// the [`BaseCode::stop`] function.
    #[must_use]
    pub fn started(&self) -> bool {
        self.0.started.into()
    }

    /// `true` if the UNDI protocol supports IPv6
    #[must_use]
    pub fn ipv6_available(&self) -> bool {
        self.0.ipv6_available.into()
    }

    /// `true` if this PXE Base Code Protocol implementation supports IPv6.
    #[must_use]
    pub fn ipv6_supported(&self) -> bool {
        self.0.ipv6_supported.into()
    }

    /// `true` if this device is currently using IPv6. This field is set by
    /// [`BaseCode::start`].
    #[must_use]
    pub fn using_ipv6(&self) -> bool {
        self.0.using_ipv6.into()
    }

    /// `true` if this PXE Base Code implementation supports Boot Integrity
    /// Services (BIS). This field is set by [`BaseCode::start`].
    #[must_use]
    pub fn bis_supported(&self) -> bool {
        self.0.bis_supported.into()
    }

    /// `true` if this device and the platform support Boot Integrity Services
    /// (BIS). This field is set by [`BaseCode::start`].
    #[must_use]
    pub fn bis_detected(&self) -> bool {
        self.0.bis_detected.into()
    }

    /// `true` for automatic ARP packet generation, `false` otherwise. This
    /// field is initialized to `true` by [`BaseCode::start`] and can be
    /// modified with [`BaseCode::set_parameters`].
    #[must_use]
    pub fn auto_arp(&self) -> bool {
        self.0.auto_arp.into()
    }

    /// This field is used to change the Client Hardware Address (chaddr) field
    /// in the DHCP and Discovery packets. Set to `true` to send the SystemGuid
    /// (if one is available). Set to `false` to send the client NIC MAC
    /// address. This field is initialized to `false` by [`BaseCode::start`]
    /// and can be modified with [`BaseCode::set_parameters`].
    #[must_use]
    pub fn send_guid(&self) -> bool {
        self.0.send_guid.into()
    }

    /// This field is initialized to `false` by [`BaseCode::start`] and set to
    /// `true` when [`BaseCode::dhcp`] completes successfully. When `true`,
    /// [`Self::dhcp_discover`] is valid. This field can also be changed by
    /// [`BaseCode::set_packets`].
    #[must_use]
    pub fn dhcp_discover_valid(&self) -> bool {
        self.0.dhcp_discover_valid.into()
    }

    /// This field is initialized to `false` by [`BaseCode::start`] and set to
    /// `true` when [`BaseCode::dhcp`] completes successfully. When `true`,
    /// [`Self::dhcp_ack`] is valid. This field can also be changed by
    /// [`BaseCode::set_packets`].
    #[must_use]
    pub fn dhcp_ack_received(&self) -> bool {
        self.0.dhcp_ack_received.into()
    }

    /// This field is initialized to `false` by [`BaseCode::start`] and set to
    /// `true` when [`BaseCode::dhcp`] completes successfully and a proxy DHCP
    /// offer packet was received. When `true`, [`Self::proxy_offer`] is valid.
    /// This field can also be changed by [`BaseCode::set_packets`].
    #[must_use]
    pub fn proxy_offer_received(&self) -> bool {
        self.0.proxy_offer_received.into()
    }

    /// When `true`, [`Self::pxe_discover`] is valid. This field is set to
    /// `false` by [`BaseCode::start`] and [`BaseCode::dhcp`], and can be set
    /// to `true` or `false` by [`BaseCode::discover`] and
    /// [`BaseCode::set_packets`].
    #[must_use]
    pub fn pxe_discover_valid(&self) -> bool {
        self.0.pxe_discover_valid.into()
    }

    /// When `true`, [`Self::pxe_reply`] is valid. This field is set to `false`
    /// by [`BaseCode::start`] and [`BaseCode::dhcp`], and can be set to `true`
    /// or `false` by [`BaseCode::discover`] and [`BaseCode::set_packets`].
    #[must_use]
    pub fn pxe_reply_received(&self) -> bool {
        self.0.pxe_reply_received.into()
    }

    /// When `true`, [`Self::pxe_bis_reply`] is valid. This field is set to
    /// `false` by [`BaseCode::start`] and [`BaseCode::dhcp`], and can be set
    /// to `true` or `false` by the [`BaseCode::discover`] and
    /// [`BaseCode::set_packets`].
    #[must_use]
    pub fn pxe_bis_reply_received(&self) -> bool {
        self.0.pxe_bis_reply_received.into()
    }

    /// Indicates whether [`Self::icmp_error`] has been updated. This field is
    /// reset to `false` by [`BaseCode::start`], [`BaseCode::dhcp`],
    /// [`BaseCode::discover`],[`BaseCode::udp_read`], [`BaseCode::udp_write`],
    /// [`BaseCode::arp`] and any of the TFTP/MTFTP operations. If an ICMP
    /// error is received, this field will be set to `true` after
    /// [`Self::icmp_error`] is updated.
    #[must_use]
    pub fn icmp_error_received(&self) -> bool {
        self.0.icmp_error_received.into()
    }

    /// Indicates whether [`Self::tftp_error`] has been updated. This field is
    /// reset to `false` by [`BaseCode::start`] and any of the TFTP/MTFTP
    /// operations. If a TFTP error is received, this field will be set to
    /// `true` after [`Self::tftp_error`] is updated.
    #[must_use]
    pub fn tftp_error_received(&self) -> bool {
        self.0.tftp_error_received.into()
    }

    /// When `false`, callbacks will not be made. When `true`, make callbacks
    /// to the PXE Base Code Callback Protocol. This field is reset to `false`
    /// by [`BaseCode::start`] if the PXE Base Code Callback Protocol is not
    /// available. It is reset to `true` by [`BaseCode::start`] if the PXE Base
    /// Code Callback Protocol is available.
    #[must_use]
    pub fn make_callbacks(&self) -> bool {
        self.0.make_callbacks.into()
    }

    /// The "time to live" field of the IP header. This field is initialized to
    /// `16` by [`BaseCode::start`] and can be modified by
    /// [`BaseCode::set_parameters`].
    #[must_use]
    pub const fn ttl(&self) -> u8 {
        self.0.ttl
    }

    /// The type of service field of the IP header. This field is initialized
    /// to `0` by [`BaseCode::start`], and can be modified with
    /// [`BaseCode::set_parameters`].
    #[must_use]
    pub const fn tos(&self) -> u8 {
        self.0.tos
    }

    /// The device’s current IP address. This field is initialized to a zero
    /// address by Start(). This field is set when [`BaseCode::dhcp`] completes
    /// successfully. This field can also be set by
    /// [`BaseCode::set_station_ip`]. This field must be set to a valid IP
    /// address by either [`BaseCode::dhcp`] or [`BaseCode::set_station_ip`]
    /// before [`BaseCode::discover`], [`BaseCode::udp_read`],
    /// [`BaseCode::udp_write`], [`BaseCode::arp`] and any of the TFTP/MTFTP
    /// operations are called.
    #[must_use]
    pub fn station_ip(&self) -> IpAddr {
        let efi_ip = self.0.station_ip;
        // SAFETY: We trust that `using_ipv6()` tells the truth.
        unsafe { efi_ip.into_core_addr(self.using_ipv6()) }
    }

    /// The device's current subnet mask. This field is initialized to a zero
    /// address by [`BaseCode::start`]. This field is set when
    /// [`BaseCode::dhcp`] completes successfully. This field can also be set
    /// by [`BaseCode::set_station_ip`]. This field must be set to a valid
    /// subnet mask by either [`BaseCode::dhcp`] or
    /// [`BaseCode::set_station_ip`] before [`BaseCode::discover`],
    /// [`BaseCode::udp_read`], [`BaseCode::udp_write`],
    /// [`BaseCode::arp`] or any of the TFTP/MTFTP operations are called.
    #[must_use]
    pub fn subnet_mask(&self) -> IpAddr {
        let efi_ip = self.0.subnet_mask;
        // SAFETY: We trust that `using_ipv6()` tells the truth.
        unsafe { efi_ip.into_core_addr(self.using_ipv6()) }
    }

    /// Cached DHCP Discover packet. This field is zero-filled by the
    /// [`BaseCode::start`] function, and is set when [`BaseCode::dhcp`]
    /// completes successfully. The contents of this field can replaced by
    /// [`BaseCode::set_packets`].
    #[must_use]
    pub const fn dhcp_discover(&self) -> &Packet {
        &self.0.dhcp_discover
    }

    /// Cached DHCP Ack packet. This field is zero-filled by
    /// [`BaseCode::start`], and is set when [`BaseCode::dhcp`] completes
    /// successfully. The contents of this field can be replaced by
    /// [`BaseCode::set_packets`].
    #[must_use]
    pub const fn dhcp_ack(&self) -> &Packet {
        &self.0.dhcp_ack
    }

    /// Cached Proxy Offer packet. This field is zero-filled by
    /// [`BaseCode::start`], and is set when [`BaseCode::dhcp`] completes
    /// successfully. The contents of this field can be replaced by
    /// [`BaseCode::set_packets`].
    #[must_use]
    pub const fn proxy_offer(&self) -> &Packet {
        &self.0.proxy_offer
    }

    /// Cached PXE Discover packet. This field is zero-filled by
    /// [`BaseCode::start`], and is set when [`BaseCode::discover`] completes
    /// successfully. The contents of this field can be replaced by
    /// [`BaseCode::set_packets`].
    #[must_use]
    pub const fn pxe_discover(&self) -> &Packet {
        &self.0.pxe_discover
    }

    /// Cached PXE Reply packet. This field is zero-filled by
    /// [`BaseCode::start`], and is set when [`BaseCode::discover`] completes
    /// successfully. The contents of this field can be replaced by the
    /// [`BaseCode::set_packets`] function.
    #[must_use]
    pub const fn pxe_reply(&self) -> &Packet {
        &self.0.pxe_reply
    }

    /// Cached PXE BIS Reply packet. This field is zero-filled by
    /// [`BaseCode::start`], and is set when [`BaseCode::discover`] completes
    /// successfully. This field can be replaced by [`BaseCode::set_packets`].
    #[must_use]
    pub const fn pxe_bis_reply(&self) -> &Packet {
        &self.0.pxe_bis_reply
    }

    /// The current IP receive filter settings. The receive filter is disabled
    /// and the number of IP receive filters is set to zero by
    /// [`BaseCode::start`], and is set by [`BaseCode::set_ip_filter`].
    #[must_use]
    pub const fn ip_filter(&self) -> &IpFilter {
        &self.0.ip_filter
    }

    /// Cached ARP entries.
    #[must_use]
    pub const fn arp_cache(&self) -> &[ArpEntry] {
        self.0
            .arp_cache
            .split_at(usize_from_u32(self.0.arp_cache_entries))
            .0
    }

    /// The number of valid entries in the current route table. This field is
    /// reset to zero by [`BaseCode::start`].
    #[must_use]
    pub const fn route_table_entries(&self) -> u32 {
        self.0.route_table_entries
    }

    /// Array of route table entries.
    #[must_use]
    pub const fn route_table(&self) -> &[RouteEntry] {
        self.0
            .route_table
            .split_at(usize_from_u32(self.0.route_table_entries))
            .0
    }

    /// ICMP error packet. This field is updated when an ICMP error is received
    /// and is undefined until the first ICMP error is received. This field is
    /// zero-filled by [`BaseCode::start`].
    #[must_use]
    pub const fn icmp_error(&self) -> &IcmpError {
        &self.0.icmp_error
    }

    /// TFTP error packet. This field is updated when a TFTP error is received
    /// and is undefined until the first TFTP error is received. This field is
    /// zero-filled by the [`BaseCode::start`] function.
    #[must_use]
    pub const fn tftp_error(&self) -> &TftpError {
        &self.0.tftp_error
    }
}

/// Returned by [`BaseCode::tftp_read_dir`].
#[expect(missing_docs)]
#[derive(Debug)]
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
#[expect(missing_docs)]
#[derive(Debug)]
pub struct MtftpFileInfo<'a> {
    pub filename: &'a CStr8,
    pub ip_address: IpAddr,
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

impl Display for ReadDirParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl core::error::Error for ReadDirParseError {}
