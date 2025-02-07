// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{guid, Guid, Status};
use core::ffi::c_void;

newtype_enum! {
    pub enum TlsConfigDataType: i32 => {
        HOST_PUBLIC_CERT     = 0,
        HOST_PRIVATE_KEY     = 1,
        CA_CERTIFICATE       = 2,
        CERT_REVOCATION_LIST = 3,
        MAXIMUM              = 4,
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TlsConfigurationProtocol {
    pub set_data: unsafe extern "efiapi" fn(
        this: *mut Self,
        typ: TlsConfigDataType,
        data: *const c_void,
        size: usize,
    ) -> Status,

    pub get_data: unsafe extern "efiapi" fn(
        this: *const Self,
        typ: TlsConfigDataType,
        data: *mut c_void,
        size: *mut usize,
    ) -> Status,
}

impl TlsConfigurationProtocol {
    pub const GUID: Guid = guid!("1682fe44-bd7a-4407-b7c7-dca37ca3922d");
    pub const SERVICE_BINDING_GUID: Guid = guid!("952cb795-ff36-48cf-a249-4df486d6ab8d");
}
