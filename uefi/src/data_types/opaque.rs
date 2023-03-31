/// Create an opaque struct suitable for use as an FFI pointer.
///
/// The internal representation uses the recommendation in the [nomicon].
///
/// [nomicon]: https://doc.rust-lang.org/stable/nomicon/ffi.html#representing-opaque-structs
#[macro_export]
macro_rules! opaque_type {
    (
        $(#[$struct_attrs:meta])*
        $struct_vis:vis struct $struct_name:ident;
    ) => {
        // Create the struct with the fields recommended by the nomicon.
        $(#[$struct_attrs])*
        #[repr(C)]
        $struct_vis struct $struct_name {
            _data: [u8; 0],
            _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
        }

        // Impl Debug, just show the struct name.
        impl core::fmt::Debug for $struct_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($struct_name)).finish()
            }
        }
    }
}
