use crate::{guid, Char16, Char8, Event, Guid, Ipv4Address, Ipv6Address, Status};
use core::ffi::c_void;
use core::fmt::{self, Debug, Formatter};
use core::ptr;

#[derive(Debug, Default)]
#[repr(C)]
pub struct HttpConfigData {
    pub http_version: HttpVersion,
    pub time_out_millisec: u32,
    pub local_addr_is_ipv6: bool,
    pub access_point: HttpAccessPoint,
}

newtype_enum! {
    #[derive(Default)]
    pub enum HttpVersion: i32 => {
        HTTP_VERSION_10 = 0,
        HTTP_VERSION_11 = 1,
        HTTP_VERSION_UNSUPPORTED = 2,
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(C)]
pub struct HttpV4AccessPoint {
    pub use_default_addr: bool,
    pub local_address: Ipv4Address,
    pub local_subnet: Ipv4Address,
    pub local_port: u16,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(C)]
pub struct HttpV6AccessPoint {
    pub local_address: Ipv6Address,
    pub local_port: u16,
}

#[repr(C)]
pub union HttpAccessPoint {
    pub ipv4_node: *const HttpV4AccessPoint,
    pub ipv6_node: *const HttpV6AccessPoint,
}

impl Debug for HttpAccessPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // This is a union type, so we can't access the internal data.
        f.debug_struct("HttpAccessPoint").finish()
    }
}

impl Default for HttpAccessPoint {
    fn default() -> Self {
        Self {
            ipv4_node: ptr::null(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct HttpToken {
    pub event: Event,
    pub status: Status,
    pub message: *mut HttpMessage,
}

impl Default for HttpToken {
    fn default() -> Self {
        Self {
            event: ptr::null_mut(),
            status: Status::SUCCESS,
            message: ptr::null_mut(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct HttpMessage {
    pub data: HttpRequestOrResponse,
    pub header_count: usize,
    pub header: *mut HttpHeader,
    pub body_length: usize,
    pub body: *mut c_void,
}

impl Default for HttpMessage {
    fn default() -> Self {
        Self {
            data: HttpRequestOrResponse::default(),
            header_count: 0,
            header: ptr::null_mut(),
            body_length: 0,
            body: ptr::null_mut(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct HttpRequestData {
    pub method: HttpMethod,
    pub url: *const Char16,
}

impl Default for HttpRequestData {
    fn default() -> Self {
        Self {
            method: HttpMethod::default(),
            url: ptr::null(),
        }
    }
}

newtype_enum! {
    #[derive(Default)]
    pub enum HttpMethod: i32 => {
        GET     = 0,
        POST    = 1,
        PATCH   = 2,
        OPTIONS = 3,
        CONNECT = 4,
        HEAD    = 5,
        PUT     = 6,
        DELETE  = 7,
        TRACE   = 8,
        MAX     = 9,
    }
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct HttpResponseData {
    pub status_code: HttpStatusCode,
}

#[repr(C)]
pub union HttpRequestOrResponse {
    pub request: *const HttpRequestData,
    pub response: *const HttpResponseData,
}

impl Debug for HttpRequestOrResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // This is a union type, so we can't access the internal data.
        f.debug_struct("RequestOrResponse").finish()
    }
}

impl Default for HttpRequestOrResponse {
    fn default() -> Self {
        Self {
            request: ptr::null(),
        }
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct HttpHeader {
    pub field_name: *const Char8,
    pub field_value: *const Char8,
}

impl Default for HttpHeader {
    fn default() -> Self {
        Self {
            field_name: ptr::null(),
            field_value: ptr::null(),
        }
    }
}

newtype_enum! {
    #[derive(Default)]
    pub enum HttpStatusCode: i32 => {
        STATUS_UNSUPPORTED = 0,
        STATUS_100_CONTINUE = 1,
        STATUS_101_SWITCHING_PROTOCOLS = 2,
        STATUS_200_OK = 3,
        STATUS_201_CREATED = 4,
        STATUS_202_ACCEPTED = 5,
        STATUS_203_NON_AUTHORITATIVE_INFORMATION = 6,
        STATUS_204_NO_CONTENT = 7,
        STATUS_205_RESET_CONTENT = 8,
        STATUS_206_PARTIAL_CONTENT = 9,
        STATUS_300_MULTIPLE_CHOICES = 10,
        STATUS_301_MOVED_PERMANENTLY = 11,
        STATUS_302_FOUND = 12,
        STATUS_303_SEE_OTHER = 13,
        STATUS_304_NOT_MODIFIED = 14,
        STATUS_305_USE_PROXY = 15,
        STATUS_307_TEMPORARY_REDIRECT = 16,
        STATUS_400_BAD_REQUEST = 17,
        STATUS_401_UNAUTHORIZED = 18,
        STATUS_402_PAYMENT_REQUIRED = 19,
        STATUS_403_FORBIDDEN = 20,
        STATUS_404_NOT_FOUND = 21,
        STATUS_405_METHOD_NOT_ALLOWED = 22,
        STATUS_406_NOT_ACCEPTABLE = 23,
        STATUS_407_PROXY_AUTHENTICATION_REQUIRED = 24,
        STATUS_408_REQUEST_TIME_OUT = 25,
        STATUS_409_CONFLICT = 26,
        STATUS_410_GONE = 27,
        STATUS_411_LENGTH_REQUIRED = 28,
        STATUS_412_PRECONDITION_FAILED = 29,
        STATUS_413_REQUEST_ENTITY_TOO_LARGE = 30,
        STATUS_414_REQUEST_URI_TOO_LARGE = 31,
        STATUS_415_UNSUPPORTED_MEDIA_TYPE = 32,
        STATUS_416_REQUESTED_RANGE_NOT_SATISFIED = 33,
        STATUS_417_EXPECTATION_FAILED = 34,
        STATUS_500_INTERNAL_SERVER_ERROR = 35,
        STATUS_501_NOT_IMPLEMENTED = 36,
        STATUS_502_BAD_GATEWAY = 37,
        STATUS_503_SERVICE_UNAVAILABLE = 38,
        STATUS_504_GATEWAY_TIME_OUT = 39,
        STATUS_505_VERSION_NOT_SUPPORTED = 40,
        STATUS_308_PERMANENT_REDIRECT = 41,
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct HttpProtocol {
    pub get_mode_data:
        unsafe extern "efiapi" fn(this: *const Self, config_data: *mut HttpConfigData) -> Status,
    pub configure:
        unsafe extern "efiapi" fn(this: *mut Self, config_data: *const HttpConfigData) -> Status,
    pub request: unsafe extern "efiapi" fn(this: *mut Self, token: *mut HttpToken) -> Status,
    pub cancel: unsafe extern "efiapi" fn(this: *mut Self, token: *mut HttpToken) -> Status,
    pub response: unsafe extern "efiapi" fn(this: *mut Self, token: *mut HttpToken) -> Status,
    pub poll: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
}

impl HttpProtocol {
    pub const GUID: Guid = guid!("7a59b29b-910b-4171-8242-a85a0df25b5b");
    pub const SERVICE_BINDING_GUID: Guid = guid!("bdc8e6af-d9bc-4379-a72a-e0c4e75dae1c");
}
