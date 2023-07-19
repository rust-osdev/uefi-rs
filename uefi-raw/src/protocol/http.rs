use crate::{guid, Char16, Char8, Event, Guid, Status};
use core::ffi::c_void;

#[repr(C)]
pub struct ConfigData {
    pub http_version: Version,
    pub timeout_ms: u32,
    pub local_addr_is_ipv6: bool,
    pub access_point: AccessPoint,
}

newtype_enum! {
    pub enum Version: i32 => {
        HTTP_VERSION_10 = 0,
        HTTP_VERSION_11 = 1,
        HTTP_VERSION_UNSUPPORTED = 2,
    }
}

#[repr(C)]
pub struct V4AccessPoint {
    pub use_default_addr: bool,
    pub local_address: [u8; 4],
    pub local_subnet: [u8; 4],
    pub local_port: u16,
}

#[repr(C)]
pub struct V6AccessPoint {
    pub local_address: [u8; 16],
    pub local_port: u16,
}

#[repr(C)]
pub union AccessPoint {
    pub ipv4_node: *const V4AccessPoint,
    pub ipv6_node: *const V6AccessPoint,
}

#[derive(Debug)]
#[repr(C)]
pub struct Token {
    pub event: Event,
    pub status: Status,
    pub message: *mut Message,
}

#[repr(C)]
pub struct Message {
    pub data: RequestOrResponse,
    pub header_count: usize,
    pub header: *mut Header,
    pub body_length: usize,
    pub body: *mut c_void,
}

#[repr(C)]
pub struct RequestData {
    pub method: Method,
    pub url: *const Char16,
}

newtype_enum! {
    pub enum Method: i32 => {
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

#[repr(C)]
pub struct ResponseData {
    pub status_code: Code,
}

#[repr(C)]
pub union RequestOrResponse {
    pub request: *const RequestData,
    pub response: *const ResponseData,
}

#[derive(Clone)]
#[repr(C)]
pub struct Header {
    pub field_name: *const Char8,
    pub field_value: *const Char8,
}

newtype_enum! {
    pub enum Code: i32 => {
        UNSUPPORTED                      =  0,
        /// 100 Continue
        CONTINUE                         =  1,
        /// 101 Switching Protocols
        SWITCHING_PROTOCOLS              =  2,
        /// 200 OK
        OK                               =  3,
        /// 201 Created
        CREATED                          =  4,
        /// 202 Accepted
        ACCEPTED                         =  5,
        /// 203 Non Authoritative Information
        NON_AUTHORITATIVE_INFORMATION    =  6,
        /// 204 No Content
        NO_CONTENT                       =  7,
        /// 205 Reset Content
        RESET_CONTENT                    =  8,
        /// 206 Partial Content
        PARTIAL_CONTENT                  =  9,
        /// 300 Multiple Choices
        MULTIPLE_CHOICES                 = 10,
        /// 301 Moved Permanently
        MOVED_PERMANENTLY                = 11,
        /// 302 Found
        FOUND                            = 12,
        /// 303 See Other
        SEE_OTHER                        = 13,
        /// 304 Not Modified
        NOT_MODIFIED                     = 14,
        /// 305 Use Proxy
        USE_PROXY                        = 15,
        /// 307 Temporary Redirect
        TEMPORARY_REDIRECT               = 16,
        /// 400 Bad Request
        BAD_REQUEST                      = 17,
        /// 401 Unauthorized
        UNAUTHORIZED                     = 18,
        /// 402 Payment Required
        PAYMENT_REQUIRED                 = 19,
        /// 403 Forbidden
        FORBIDDEN                        = 20,
        /// 404 Not Found
        NOT_FOUND                        = 21,
        /// 405 Method Not Allowed
        METHOD_NOT_ALLOWED               = 22,
        /// 406 Not Acceptable
        NOT_ACCEPTABLE                   = 23,
        /// 407 Proxy Authentication Required
        PROXY_AUTHENTICATION_REQUIRED    = 24,
        /// 408 Request Time Out
        REQUEST_TIME_OUT                 = 25,
        /// 409 Conflict
        CONFLICT                         = 26,
        /// 410 Gone
        GONE                             = 27,
        /// 411 Length Required
        LENGTH_REQUIRED                  = 28,
        /// 412 Precondition Failed
        PRECONDITION_FAILED              = 29,
        /// 413 Request Entity Too Large
        REQUEST_ENTITY_TOO_LARGE         = 30,
        /// 414 Request URI Too Large
        REQUEST_URI_TOO_LARGE            = 31,
        /// 415 Unsupported Media Type
        UNSUPPORTED_MEDIA_TYPE           = 32,
        /// 416 Requested Range Not Satisfied
        REQUESTED_RANGE_NOT_SATISFIED    = 33,
        /// 417 Expectation Failed
        EXPECTATION_FAILED               = 34,
        /// 500 Internal Server Error
        INTERNAL_SERVER_ERROR            = 35,
        /// 501 Not Implemented
        NOT_IMPLEMENTED                  = 36,
        /// 502 Bad Gateway
        BAD_GATEWAY                      = 37,
        /// 503 Service Unavailable
        SERVICE_UNAVAILABLE              = 38,
        /// 504 Gateway Time Out
        GATEWAY_TIME_OUT                 = 39,
        /// 505 HTTP Version Not Supported
        HTTP_VERSION_NOT_SUPPORTED       = 40,
        /// 308 Permanent Redirect
        PERMANENT_REDIRECT               = 41,
    }
}

/// The EFI HTTP protocol is designed to be used by EFI drivers and applications
/// to create and transmit HTTP Requests, as well as handle HTTP responses that
/// are returned by a remote host. This EFI protocol uses and relies on an
/// underlying EFI TCP protocol.
#[repr(C)]
pub struct HttpProtocol {
    /// Returns the operational parameters for the current HTTP child instance.
    ///
    /// The GetModeData() function is used to read the current mode data
    /// (operational parameters) for this HTTP protocol instance.
    pub get_mode_data:
        unsafe extern "efiapi" fn(this: &Self, config_data: *mut ConfigData) -> Status,

    /// Initialize or brutally reset the operational parameters for this EFI
    /// HTTP instance.
    ///
    /// The Configure() function does the following:
    ///
    ///     - When HttpConfigData is not NULL Initialize this EFI HTTP instance by
    ///       configuring timeout, local address, port, etc.
    ///
    ///     - When HttpConfigData is NULL, reset this EFI HTTP instance by
    ///       closing all active connections with remote hosts, canceling all
    ///       asynchronous tokens, and flush request and response buffers
    ///       without informing the appropriate hosts.
    ///
    /// No other EFI HTTP function can be executed by this instance until the
    /// Configure() function is executed and returns successfully.
    pub configure: unsafe extern "efiapi" fn(this: &Self, config_data: *const ConfigData) -> Status,

    /// The Request() function queues an HTTP request to this HTTP instance,
    /// similar to Transmit() function in the EFI TCP driver. When the HTTP
    /// request is sent successfully, or if there is an error, Status in token
    /// will be updated and Event will be signaled.
    ///
    /// The HTTP driver will prepare a request string from the information
    /// contained in and queue it to the underlying TCP instance to be sent to
    /// the remote host. Typically, all fields in the structure will contain
    /// content (except Body and BodyLength when HTTP method is not POST or
    /// PUT), but there is a special case when using PUT or POST to send large
    /// amounts of data. Depending on the size of the data, it may not be able
    /// to be stored in a contiguous block of memory, so the data will need to
    /// be provided in chunks. In this case, if Body is not NULL and BodyLength
    /// is non-zero and all other fields are NULL or 0, the HTTP driver will
    /// queue the data to be sent to the last remote host that a token was
    /// successfully sent. If no previous token was sent successfully, this
    /// function will return EFI_INVALID_PARAMETER.
    ///
    /// The HTTP driver is expected to close existing (if any) underlying TCP
    /// instance and create new TCP instance if the host name in the request URL
    /// is different from previous calls to Request(). This is consistent with
    /// RFC 2616 recommendation that HTTP clients should attempt to maintain an
    /// open TCP connection between client and host.
    pub request: unsafe extern "efiapi" fn(this: &Self, token: *mut Token) -> Status,

    /// Abort an asynchronous HTTP request or response token.
    ///
    /// The Cancel() function aborts a pending HTTP request or response
    /// transaction. If Token is not NULL and the token is in transmit or
    /// receive queues when it is being cancelled, its Token->Status will be set
    /// to EFI_ABORTED and then Token->Event will be signaled. If the token is
    /// not in one of the queues, which usually means that the asynchronous
    /// operation has completed, EFI_NOT_FOUND is returned. If Token is NULL,
    /// all asynchronous tokens issued by Request() or Response() will be
    /// aborted.
    pub cancel: unsafe extern "efiapi" fn(this: &Self, token: *mut Token) -> Status,

    /// The Response() function queues an HTTP response to this HTTP instance,
    /// similar to Receive() function in the EFI TCP driver. When the HTTP
    /// response is received successfully, or if there is an error, Status in
    /// token will be updated and Event will be signaled.
    ///
    /// The HTTP driver will queue a receive token to the underlying TCP
    /// instance. When data is received in the underlying TCP instance, the data
    /// will be parsed and Token will be populated with the response data. If
    /// the data received from the remote host contains an incomplete or invalid
    /// HTTP header, the HTTP driver will continue waiting (asynchronously) for
    /// more data to be sent from the remote host before signaling Event in
    /// Token.
    ///
    /// It is the responsibility of the caller to allocate a buffer for Body and
    /// specify the size in BodyLength. If the remote host provides a response
    /// that contains a content body, up to BodyLength bytes will be copied from
    /// the receive buffer into Body and BodyLength will be updated with the
    /// amount of bytes received and copied to Body. This allows the client to
    /// download a large file in chunks instead of into one contiguous block of
    /// memory. Similar to HTTP request, if Body is not NULL and BodyLength is
    /// non-zero and all other fields are NULL or 0, the HTTP driver will queue
    /// a receive token to underlying TCP instance. If data arrives in the
    /// receive buffer, up to BodyLength bytes of data will be copied to Body.
    /// The HTTP driver will then update BodyLength with the amount of bytes
    /// received and copied to Body.
    ///
    /// If the HTTP driver does not have an open underlying TCP connection with
    /// the host specified in the response URL, Response() will return
    /// EFI_ACCESS_DENIED. This is consistent with RFC 2616 recommendation that
    /// HTTP clients should attempt to maintain an open TCP connection between
    /// client and host.
    pub response: unsafe extern "efiapi" fn(this: &Self, token: *mut Token) -> Status,

    /// Polls for incoming data packets and processes outgoing data packets.
    ///
    /// The Poll() function can be used by network drivers and applications to
    /// increase the rate that data packets are moved between the communication
    /// devices and the transmit and receive queues. In some systems, the
    /// periodic timer event in the managed network driver may not poll the
    /// underlying communications device fast enough to transmit and/or receive
    /// all data packets without missing incoming packets or dropping outgoing
    /// packets. Drivers and applications that are experiencing packet loss
    /// should try calling the Poll() function more often.
    pub poll: unsafe extern "efiapi" fn(this: &Self) -> Status,
}

impl HttpProtocol {
    pub const GUID: Guid = guid!("7a59b29b-910b-4171-8242-a85a0df25b5b");
    pub const SERVICE_GUID: Guid = guid!("bdc8e6af-d9bc-4379-a72a-e0c4e75dae1c");
}
