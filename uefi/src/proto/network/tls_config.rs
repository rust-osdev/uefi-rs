//! `TLS Config` protocol.

use crate::proto::unsafe_protocol;
use crate::{Result, Status, StatusExt};
use core::ffi::c_void;
use core::mem;
use uefi_raw::protocol::tls_config::{DataType, TlsConfigProtocol};

/// TLS Config protocol
#[unsafe_protocol(
    TlsConfigProtocol::GUID,
    TlsConfigProtocol::SERVICE_GUID,
    TlsConfigProtocol
)]
pub struct TlsConfig<'a> {
    proto: &'a mut TlsConfigProtocol,
}

impl From<*mut TlsConfigProtocol> for TlsConfig<'_> {
    fn from(proto: *mut TlsConfigProtocol) -> Self {
        Self {
            proto: unsafe { &mut *proto },
        }
    }
}

impl TlsConfig<'_> {
    /// Add a CA certificate to the trust store for either client or
    /// server verification. Both DER and PEM contents are accepted.
    /// This method is idempotent, so providing the same certificate
    /// twice will not error.
    ///
    /// CA certificates are also (optionally) provided by the platform
    /// in the [`EFI_TLS_CA_CERTIFICATE_VARIABLE`] efivar, which is a
    /// database in the [`EFI_SIGNATURE_LIST`] format. Even if the CA
    /// trust store is reset, the first HTTPS connection (not raw TLS)
    /// will populate it with the contents of this variable.
    ///
    /// [`EFI_SIGNATURE_LIST`]: https://github.com/tianocore/edk2/blob/edk2-stable202305/MdePkg/Include/Guid/ImageAuthentication.h#L63-L88
    /// [`EFI_TLS_CA_CERTIFICATE_VARIABLE`]: https://github.com/tianocore/edk2/blob/edk2-stable202305/NetworkPkg/Include/Guid/TlsAuthentication.h#L19
    pub fn add_ca(&mut self, data: impl AsRef<[u8]>) -> Result {
        self.set_ref(DataType::CA_CERTIFICATE, data.as_ref())
    }
}

// Wrappers to raw protocol
impl TlsConfig<'_> {
    #[allow(unused)]
    fn set<T>(&mut self, data_type: DataType, data: T) -> Result {
        unsafe {
            (self.proto.set_data)(
                &self.proto,
                data_type,
                &data as *const _ as *const c_void,
                mem::size_of_val(&data),
            )
        }
        .to_result()
    }

    fn set_ref<T: ?Sized>(&mut self, data_type: DataType, data: &T) -> Result {
        let size = mem::size_of_val(data);
        unsafe {
            (self.proto.set_data)(
                &self.proto,
                data_type,
                data as *const _ as *const c_void,
                size,
            )
        }
        .to_result()
    }

    #[allow(unused)]
    fn get<T>(&mut self, data_type: DataType) -> Result<T, usize> {
        let mut size: usize = mem::size_of::<T>();
        let mut data: T = unsafe { mem::zeroed() };
        self.get_with(data_type, &mut size, &mut data).map(|_| data)
    }

    #[allow(unused)]
    fn get_with<T>(
        &mut self,
        data_type: DataType,
        data_size: &mut usize,
        data: &mut T,
    ) -> Result<(), usize> {
        unsafe {
            (self.proto.get_data)(
                &self.proto,
                data_type,
                data as *mut _ as *mut c_void,
                data_size,
            )
        }
        .to_result_with_err(|status| match status {
            Status::BUFFER_TOO_SMALL => *data_size,
            _ => 0,
        })
    }
}
