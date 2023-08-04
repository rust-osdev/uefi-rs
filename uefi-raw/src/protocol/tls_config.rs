use crate::{guid, Guid, Status};
use core::ffi::c_void;

newtype_enum! {
    pub enum DataType: i32 => {
        /// Local host configuration data: public certificate data. This data
        /// should be DER-encoded binary X.509 certificate or PEM-encoded X.509
        /// certificate.
        HOST_PUBLIC_CERT     = 0,
        /// Local host configuration data: private key data. This data should
        /// be PEM-encoded RSA or PKCS#8 private key.
        HOST_PRIVATE_KEY     = 1,
        /// CA certificate to verify peer. This data should be DER-encoded
        /// binary X.509 certificate or PEM-encoded X.509 certificate.
        CA_CERTIFICATE       = 2,
        /// CA-supplied Certificate Revocation List data. This data should be
        /// DER-encoded CRL data.
        CERT_REVOCATION_LIST = 3,
        MAX                  = 4,
    }
}

#[repr(C)]
pub struct TlsConfigProtocol {
    /// The SetData() function sets TLS configuration to non-volatile storage or
    /// volatile storage.
    pub set_data: unsafe extern "efiapi" fn(
        this: &Self,
        typ: DataType,
        data: *const c_void,
        size: usize,
    ) -> Status,

    /// The GetData() function gets TLS configuration.
    pub get_data: unsafe extern "efiapi" fn(
        this: &Self,
        typ: DataType,
        data: *mut c_void,
        size: *mut usize,
    ) -> Status,
}

impl TlsConfigProtocol {
    pub const GUID: Guid = guid!("1682fe44-bd7a-4407-b7c7-dca37ca3922d");
    pub const SERVICE_GUID: Guid = guid!("952cb795-ff36-48cf-a249-4df486d6ab8d");
}
