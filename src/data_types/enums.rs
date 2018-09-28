//! This module provides tooling that facilitates dealing with C-style enums
//!
//! C-style enums and Rust-style enums are quite different. There are things
//! which one allows, but not the others, and vice versa. In an FFI context, two
//! aspects of C-style enums are particularly bothersome to us:
//!
//! - They allow a caller to send back an unknown enum variant. In Rust, the
//!   mere act of storing such a variant in a variable is undefined behavior.
//! - They have an implicit conversion to integers, which is often used as a
//!   more portable alternative to C bitfields or as a way to count the amount
//!   of variants of an enumerated type. Rust enums do not model this well.
//!
//! Therefore, in many cases, C enums are best modeled as newtypes of integers
//! featuring a large set of associated constants instead of as Rust enums. This
//! module provides facilities to simplify this kind of FFI.



/// Add a set of enum variants to a C enum that is modeled as an integer newtype
///
/// ```
/// pub struct UnixBool(i32);
/// newtype_enum_variants! { UnixBool => #[allow(missing_docs)] {
///     FALSE          =  0,
///     TRUE           =  1,
///     FILE_NOT_FOUND = -1,
/// }}
/// ```
#[macro_export]
macro_rules! newtype_enum_variants {
    ( $type:tt => $(#[$outer:meta])* {
        $(  $(#[$inner:meta])*
            $variant:ident = $value:expr, )*
    } ) => {
        $(#[$outer])*
        #[allow(unused)]
        impl $type {
            $(  $(#[$inner])*
                pub const $variant: $type = $type($value); )*
        }
    }
}