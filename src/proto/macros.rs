/// Implements the `Protocol` trait for a type.
/// Also marks the type as not sync and not send.
///
/// # Usage
///
/// ```rust
/// struct CustomProtocol {
///     function_pointer: extern "C" fn() -> (),
///     data: usize
/// }
///
/// impl_proto! {
///     protocol CustomProtocol {
///         GUID = 0x1234_5678, 0x9ABC 0xDEF0, [0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, 0x89];
///     }
/// }
/// ```
macro_rules! impl_proto {
    (
        protocol $p:ident {
            GUID = $a:expr, $b:expr, $c:expr, $d:expr;
        }
    ) => {
        impl $crate::proto::Protocol for $p {
            #[doc(hidden)]
            const GUID: $crate::Guid = $crate::Guid::from_values($a, $b, $c, $d);
        }

        // Most UEFI functions expect to be called on the bootstrap processor.
        impl !Send for $p {}
        // Most UEFI functions do not support multithreaded access.
        impl !Sync for $p {}
    };
}
