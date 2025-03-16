// SPDX-License-Identifier: MIT OR Apache-2.0

#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, TokenStreamExt};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, parse_quote_spanned, Error, Expr, ExprLit, ExprPath, ItemFn,
    ItemStruct, Lit, Visibility,
};

macro_rules! err {
    ($span:expr, $message:expr $(,)?) => {
        Error::new($span.span(), $message).to_compile_error()
    };
    ($span:expr, $message:expr, $($args:expr),*) => {
        Error::new($span.span(), format!($message, $($args),*)).to_compile_error()
    };
}

/// Attribute macro for marking structs as UEFI protocols.
///
/// The macro can only be applied to a struct, and takes one argument, either a
/// GUID string or the path to a `Guid` constant.
///
/// The macro implements the [`Protocol`] trait and the `unsafe` [`Identify`]
/// trait for the struct. See the [`Protocol`] trait for details of how it is
/// used.
///
/// # Safety
///
/// The caller must ensure that the correct GUID is attached to the
/// type. An incorrect GUID could lead to invalid casts and other
/// unsound behavior.
///
/// # Example
///
/// ```
/// use uefi::{Guid, Identify, guid};
/// use uefi::proto::unsafe_protocol;
///
/// #[unsafe_protocol("12345678-9abc-def0-1234-56789abcdef0")]
/// struct ExampleProtocol1 {}
///
/// const PROTO_GUID: Guid = guid!("12345678-9abc-def0-1234-56789abcdef0");
/// #[unsafe_protocol(PROTO_GUID)]
/// struct ExampleProtocol2 {}
///
/// assert_eq!(ExampleProtocol1::GUID, PROTO_GUID);
/// assert_eq!(ExampleProtocol2::GUID, PROTO_GUID);
/// ```
///
/// [`Identify`]: https://docs.rs/uefi/latest/uefi/data_types/trait.Identify.html
/// [`Protocol`]: https://docs.rs/uefi/latest/uefi/proto/trait.Protocol.html
/// [send-and-sync]: https://doc.rust-lang.org/nomicon/send-and-sync.html
#[proc_macro_attribute]
pub fn unsafe_protocol(args: TokenStream, input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(args as Expr);

    let guid_val = match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(lit), ..
        }) => {
            quote!(::uefi::guid!(#lit))
        }
        Expr::Path(ExprPath { path, .. }) => quote!(#path),
        _ => {
            return err!(
                expr,
                "macro input must be either a string literal or path to a constant"
            )
            .into()
        }
    };

    let item_struct = parse_macro_input!(input as ItemStruct);

    let ident = &item_struct.ident;
    let (impl_generics, ty_generics, where_clause) = item_struct.generics.split_for_impl();

    quote! {
        #item_struct

        unsafe impl #impl_generics ::uefi::Identify for #ident #ty_generics #where_clause {
            const GUID: ::uefi::Guid = #guid_val;
        }

        impl #impl_generics ::uefi::proto::Protocol for #ident #ty_generics #where_clause {}
    }
    .into()
}

/// Custom attribute for a UEFI executable entry point.
///
/// This attribute modifies a function to mark it as the entry point for
/// a UEFI executable. The function:
/// * Must return [`Status`].
/// * Must have zero parameters.
/// * Can optionally be `unsafe`.
///
/// The global system table pointer and global image handle will be set
/// automatically.
///
/// # Examples
///
/// ```no_run
/// #![no_main]
///
/// use uefi::prelude::*;
///
/// #[entry]
/// fn main() -> Status {
///     Status::SUCCESS
/// }
/// ```
///
/// [`Status`]: https://docs.rs/uefi/latest/uefi/struct.Status.html
#[proc_macro_attribute]
pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    // This code is inspired by the approach in this embedded Rust crate:
    // https://github.com/rust-embedded/cortex-m-rt/blob/965bf1e3291571e7e3b34834864117dc020fb391/macros/src/lib.rs#L85

    let mut errors = TokenStream2::new();

    if !args.is_empty() {
        errors.append_all(err!(
            TokenStream2::from(args),
            "Entry attribute accepts no arguments"
        ));
    }

    let mut f = parse_macro_input!(input as ItemFn);

    if let Some(ref abi) = f.sig.abi {
        errors.append_all(err!(abi, "Entry function must have no ABI modifier"));
    }
    if let Some(asyncness) = f.sig.asyncness {
        errors.append_all(err!(asyncness, "Entry function should not be async"));
    }
    if let Some(constness) = f.sig.constness {
        errors.append_all(err!(constness, "Entry function should not be const"));
    }
    if !f.sig.generics.params.is_empty() {
        errors.append_all(err!(
            f.sig.generics.params,
            "Entry function should not be generic"
        ));
    }
    if !f.sig.inputs.is_empty() {
        errors.append_all(err!(f.sig.inputs, "Entry function must have no arguments"));
    }

    // Show most errors all at once instead of one by one.
    if !errors.is_empty() {
        return errors.into();
    }

    let signature_span = f.sig.span();

    // Fill in the image handle and system table arguments automatically.
    let image_handle_ident = quote!(internal_image_handle);
    let system_table_ident = quote!(internal_system_table);
    f.sig.inputs = parse_quote_spanned!(
        signature_span=>
            #image_handle_ident: ::uefi::Handle,
            #system_table_ident: *const ::core::ffi::c_void,
    );

    // Insert code at the beginning of the entry function to set the global
    // image handle and system table pointer.
    f.block.stmts.insert(
        0,
        parse_quote! {
            unsafe {
                ::uefi::boot::set_image_handle(#image_handle_ident);
                ::uefi::table::set_system_table(#system_table_ident.cast());
            }
        },
    );

    // Set the required ABI.
    f.sig.abi = Some(parse_quote_spanned!(signature_span=> extern "efiapi"));

    // Strip any visibility modifiers.
    f.vis = Visibility::Inherited;

    let unsafety = &f.sig.unsafety;
    let fn_ident = &f.sig.ident;
    let fn_output = &f.sig.output;

    // Get the expected argument types for the main function.
    let expected_args = quote!(::uefi::Handle, *const core::ffi::c_void);

    let fn_type_check = quote_spanned! {signature_span=>
        // Cast from the function type to a function pointer with the same
        // signature first, then try to assign that to an unnamed constant with
        // the desired function pointer type.
        //
        // The cast is used to avoid an "expected fn pointer, found fn item"
        // error if the signature is wrong, since that's not what we are
        // interested in here. Instead we want to tell the user what
        // specifically in the function signature is incorrect.
        const _:
            // The expected fn pointer type.
            #unsafety extern "efiapi" fn(#expected_args) -> ::uefi::Status =
            // Cast from a fn item to a function pointer.
            #fn_ident as #unsafety extern "efiapi" fn(#expected_args) #fn_output;
    };

    let result = quote! {
        #fn_type_check

        #[export_name = "efi_main"]
        #f

    };
    result.into()
}
