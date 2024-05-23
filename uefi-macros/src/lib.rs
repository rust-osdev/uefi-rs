#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, TokenStreamExt};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Error, Expr, ExprLit, ExprPath, FnArg, Ident, ItemFn,
    ItemStruct, Lit, Pat, Visibility,
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
/// The macro takes one argument, either a GUID string or the path to a `Guid`
/// constant.
///
/// The macro can only be applied to a struct. It implements the
/// [`Protocol`] trait and the `unsafe` [`Identify`] trait for the
/// struct.
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
/// [`Identify`]: https://docs.rs/uefi/latest/uefi/trait.Identify.html
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

/// Get the name of a function's argument at `arg_index`.
fn get_function_arg_name(f: &ItemFn, arg_index: usize, errors: &mut TokenStream2) -> Option<Ident> {
    if let Some(FnArg::Typed(arg)) = f.sig.inputs.iter().nth(arg_index) {
        if let Pat::Ident(pat_ident) = &*arg.pat {
            // The argument has a valid name such as `handle` or `_handle`.
            Some(pat_ident.ident.clone())
        } else {
            // The argument is unnamed, i.e. `_`.
            errors.append_all(err!(
                arg.pat.span(),
                "Entry method's arguments must be named"
            ));
            None
        }
    } else {
        // Either there are too few arguments, or it's the wrong kind of
        // argument (e.g. `self`).
        //
        // Don't append an error in this case. The error will be caught
        // by the typecheck later on, which will give a better error
        // message.
        None
    }
}

/// Custom attribute for a UEFI executable entry point.
///
/// This attribute modifies a function to mark it as the entry point for
/// a UEFI executable. The function must have two parameters, [`Handle`]
/// and [`SystemTable<Boot>`], and return a [`Status`]. The function can
/// optionally be `unsafe`.
///
/// Due to internal implementation details the parameters must both be
/// named, so `arg` or `_arg` are allowed, but not `_`.
///
/// The [`BootServices::set_image_handle`] function will be called
/// automatically with the image [`Handle`] argument.
///
/// # Examples
///
/// ```no_run
/// #![no_main]
///
/// use uefi::prelude::*;
///
/// #[entry]
/// fn main(image: Handle, st: SystemTable<Boot>) -> Status {
///     Status::SUCCESS
/// }
/// ```
///
/// [`Handle`]: https://docs.rs/uefi/latest/uefi/data_types/struct.Handle.html
/// [`SystemTable<Boot>`]: https://docs.rs/uefi/latest/uefi/table/struct.SystemTable.html
/// [`Status`]: https://docs.rs/uefi/latest/uefi/struct.Status.html
/// [`BootServices::set_image_handle`]: https://docs.rs/uefi/latest/uefi/table/boot/struct.BootServices.html#method.set_image_handle
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
        errors.append_all(err!(abi, "Entry method must have no ABI modifier"));
    }
    if let Some(asyncness) = f.sig.asyncness {
        errors.append_all(err!(asyncness, "Entry method should not be async"));
    }
    if let Some(constness) = f.sig.constness {
        errors.append_all(err!(constness, "Entry method should not be const"));
    }
    if !f.sig.generics.params.is_empty() {
        errors.append_all(err!(
            f.sig.generics.params,
            "Entry method should not be generic"
        ));
    }

    let image_handle_ident = get_function_arg_name(&f, 0, &mut errors);
    let system_table_ident = get_function_arg_name(&f, 1, &mut errors);

    // show most errors at once instead of one by one
    if !errors.is_empty() {
        return errors.into();
    }

    let signature_span = f.sig.span();

    f.sig.abi = Some(syn::parse2(quote_spanned! (signature_span=> extern "efiapi")).unwrap());

    // allow the entry function to be unsafe (by moving the keyword around so that it actually works)
    let unsafety = &f.sig.unsafety;
    // strip any visibility modifiers
    f.vis = Visibility::Inherited;
    // Set the global image handle. If `image_handle_ident` is `None`
    // then the typecheck is going to fail anyway.
    if let Some(image_handle_ident) = image_handle_ident {
        f.block.stmts.insert(
            0,
            parse_quote! {
                unsafe {
                    #system_table_ident.boot_services().set_image_handle(#image_handle_ident);
                    ::uefi::table::set_system_table(#system_table_ident.as_ptr().cast());
                }
            },
        );
    }

    let fn_ident = &f.sig.ident;
    // Get an iterator of the function inputs types. This is needed instead of
    // directly using `sig.inputs` because patterns you can use in fn items like
    // `mut <arg>` aren't valid in fn pointers.
    let fn_inputs = f.sig.inputs.iter().map(|arg| match arg {
        FnArg::Receiver(arg) => quote!(#arg),
        FnArg::Typed(arg) => {
            let ty = &arg.ty;
            quote!(#ty)
        }
    });
    let fn_output = &f.sig.output;

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
            #unsafety extern "efiapi" fn(::uefi::Handle, ::uefi::table::SystemTable<::uefi::table::Boot>) -> ::uefi::Status =
            // Cast from a fn item to a function pointer.
            #fn_ident as #unsafety extern "efiapi" fn(#(#fn_inputs),*) #fn_output;
    };

    let result = quote! {
        #fn_type_check

        #[export_name = "efi_main"]
        #f

    };
    result.into()
}
