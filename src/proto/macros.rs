/// Implements the `Protocol` trait for a type.
/// Also marks the type as not sync and not send.
///
/// # Usage
///
/// ```rust
/// struct CustomProtocol {
///     function_pointer: extern "win64" fn() -> (),
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
        impl $crate::Identify for $p {
            #[doc(hidden)]
            // These literals aren't meant to be human-readable.
            #[allow(clippy::unreadable_literal)]
            const GUID: $crate::Guid = $crate::Guid::from_values($a, $b, $c, $d);
        }

        impl $crate::proto::Protocol for $p {}

        // Most UEFI functions expect to be called on the bootstrap processor.
        impl !Send for $p {}
        // Most UEFI functions do not support multithreaded access.
        impl !Sync for $p {}
    };
    (
        protocol $p:ident<'boot> {
            GUID = $a:expr, $b:expr, $c:expr, $d:expr;
        }
    ) => {
        impl<'boot> $crate::Identify for $p<'boot> {
            #[doc(hidden)]
            // These literals aren't meant to be human-readable.
            #[allow(clippy::unreadable_literal)]
            const GUID: $crate::Guid = $crate::Guid::from_values($a, $b, $c, $d);
        }

        impl<'boot> $crate::proto::Protocol for $p<'boot> {}

        // Most UEFI functions expect to be called on the bootstrap processor.
        impl<'boot> !Send for $p<'boot> {}
        // Most UEFI functions do not support multithreaded access.
        impl<'boot> !Sync for $p<'boot> {}
    };
}
