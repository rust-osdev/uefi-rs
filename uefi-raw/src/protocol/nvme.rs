// SPDX-License-Identifier: MIT OR Apache-2.0

use super::device_path::DevicePathProtocol;
use crate::Status;
use core::ffi::c_void;
use uguid::{guid, Guid};

bitflags::bitflags! {
    /// In an NVMe command, the `flags` field specifies which cdw (command specific word)
    /// contains a value.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(transparent)]
    pub struct NvmExpressCommandCdwValidity: u8 {
        const CDW_2  = 0x01;
        const CDW_3  = 0x02;
        const CDW_10 = 0x04;
        const CDW_11 = 0x08;
        const CDW_12 = 0x10;
        const CDW_13 = 0x20;
        const CDW_14 = 0x40;
        const CDW_15 = 0x80;
    }

    /// Represents the `EFI_NVM_EXPRESS_PASS_THRU_ATTRIBUTES_*` defines from the UEFI specification.
    ///
    /// # UEFI Specification Description
    /// Tells if the interface is for physical NVM Express controllers or logical NVM Express controllers.
    ///
    /// Drivers for non-RAID NVM Express controllers will set both the `PHYSICAL` and the `LOGICAL` bit.
    ///
    /// Drivers for RAID controllers that allow access to the underlying physical controllers will produces
    /// two protocol instances. One where the `LOGICAL` bit is set (representing the logical RAID volume),
    /// and one where the `PHYSICAL` bit is set, which can be used to access the underlying NVMe controllers.
    ///
    /// Drivers for RAID controllers that do not allow access of the underlying NVMe controllers will only
    /// produce one protocol instance for the logical RAID volume with the `LOGICAL` bit set.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(transparent)]
    pub struct NvmExpressPassThruAttributes: u32 {
        /// If this bit is set, the interface is for directly addressable namespaces.
        const PHYSICAL = 0x0001;

        /// If this bit is set, the interface is for a single logical namespace comprising multiple namespaces.
        const LOGICAL = 0x0002;

        /// If this bit is set, the interface supports both blocking and non-blocking I/O.
        /// - All interfaces must support blocking I/O, but this bit indicates that non-blocking I/O is also supported.
        const NONBLOCKIO = 0x0004;

        /// If this bit is set, the interface supports the NVM Express command set.
        const CMD_SET_NVM = 0x0008;
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct NvmExpressPassThruMode {
    pub attributes: NvmExpressPassThruAttributes,
    pub io_align: u32,
    pub nvme_version: u32,
}

/// This structure maps to the NVM Express specification Submission Queue Entry
#[derive(Debug, Default)]
#[repr(C)]
pub struct NvmExpressCommand {
    pub cdw0: u32,
    pub flags: u8,
    pub nsid: u32,
    pub cdw2: u32,
    pub cdw3: u32,
    pub cdw10: u32,
    pub cdw11: u32,
    pub cdw12: u32,
    pub cdw13: u32,
    pub cdw14: u32,
    pub cdw15: u32,
}

newtype_enum! {
    /// Type of queues an NVMe command can be placed into
    /// (Which queue a command should be placed into depends on the command)
    #[derive(Default)]
    pub enum NvmExpressQueueType: u8  => {
        /// Admin Submission Queue
        ADMIN = 0,
        /// 1) I/O Submission Queue
        IO = 1,
    }
}

/// This structure maps to the NVM Express specification Completion Queue Entry
#[derive(Debug, Default)]
#[repr(C)]
pub struct NvmExpressCompletion {
    pub dw0: u32,
    pub dw1: u32,
    pub dw2: u32,
    pub dw3: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct NvmExpressPassThruCommandPacket {
    pub command_timeout: u64,
    pub transfer_buffer: *mut c_void,
    pub transfer_length: u32,
    pub meta_data_buffer: *mut c_void,
    pub meta_data_length: u32,
    pub queue_type: NvmExpressQueueType,
    pub nvme_cmd: *const NvmExpressCommand,
    pub nvme_completion: *mut NvmExpressCompletion,
}

#[derive(Debug)]
#[repr(C)]
pub struct NvmExpressPassThruProtocol {
    pub mode: *const NvmExpressPassThruMode,
    pub pass_thru: unsafe extern "efiapi" fn(
        this: *mut Self,
        namespace_id: u32,
        packet: *mut NvmExpressPassThruCommandPacket,
        event: *mut c_void,
    ) -> Status,
    pub get_next_namespace:
        unsafe extern "efiapi" fn(this: *const Self, namespace_id: *mut u32) -> Status,
    pub build_device_path: unsafe extern "efiapi" fn(
        this: *const Self,
        namespace_id: u32,
        device_path: *mut *const DevicePathProtocol,
    ) -> Status,
    pub get_namespace: unsafe extern "efiapi" fn(
        this: *const Self,
        device_path: *const DevicePathProtocol,
        namespace_id: *mut u32,
    ) -> Status,
}

impl NvmExpressPassThruProtocol {
    pub const GUID: Guid = guid!("52c78312-8edc-4233-98f2-1a1aa5e388a5");
}
