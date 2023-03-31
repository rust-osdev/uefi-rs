#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Error, Fields, FnArg, Ident, ItemFn, ItemStruct, LitStr, Pat,
    Visibility,
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
/// The macro takes one argument, a GUID string.
///
/// The macro can only be applied to a struct, and the struct must have
/// named fields (i.e. not a unit or tuple struct). It implements the
/// [`Protocol`] trait and the `unsafe` [`Identify`] trait for the
/// struct. It also adds a hidden field that causes the struct to be
/// marked as [`!Send` and `!Sync`][send-and-sync].
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
/// use uefi::{Identify, guid};
/// use uefi::proto::unsafe_protocol;
///
/// #[unsafe_protocol("12345678-9abc-def0-1234-56789abcdef0")]
/// struct ExampleProtocol {}
///
/// assert_eq!(ExampleProtocol::GUID, guid!("12345678-9abc-def0-1234-56789abcdef0"));
/// ```
///
/// [`Identify`]: https://docs.rs/uefi/latest/uefi/trait.Identify.html
/// [`Protocol`]: https://docs.rs/uefi/latest/uefi/proto/trait.Protocol.html
/// [send-and-sync]: https://doc.rust-lang.org/nomicon/send-and-sync.html
#[proc_macro_attribute]
pub fn unsafe_protocol(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse `args` as a GUID string.
    let (time_low, time_mid, time_high_and_version, clock_seq_and_variant, node) =
        match parse_guid(parse_macro_input!(args as LitStr)) {
            Ok(data) => data,
            Err(tokens) => return tokens.into(),
        };

    let item_struct = parse_macro_input!(input as ItemStruct);

    let ident = &item_struct.ident;
    let struct_attrs = &item_struct.attrs;
    let struct_vis = &item_struct.vis;
    let struct_fields = if let Fields::Named(struct_fields) = &item_struct.fields {
        &struct_fields.named
    } else {
        return err!(item_struct, "Protocol struct must used named fields").into();
    };
    let struct_generics = &item_struct.generics;
    let (impl_generics, ty_generics, where_clause) = item_struct.generics.split_for_impl();

    quote! {
        #(#struct_attrs)*
        #struct_vis struct #ident #struct_generics {
            // Add a hidden field with `PhantomData` of a raw
            // pointer. This has the implicit side effect of making the
            // struct !Send and !Sync.
            _no_send_or_sync: ::core::marker::PhantomData<*const u8>,
            #struct_fields
        }

        unsafe impl #impl_generics ::uefi::Identify for #ident #ty_generics #where_clause {
            const GUID: ::uefi::Guid = ::uefi::Guid::from_values(
                #time_low,
                #time_mid,
                #time_high_and_version,
                #clock_seq_and_variant,
                #node,
            );
        }

        impl #impl_generics ::uefi::proto::Protocol for #ident #ty_generics #where_clause {}
    }
    .into()
}

/// Create a `Guid` at compile time.
///
/// # Example
///
/// ```
/// use uefi::{guid, Guid};
/// const EXAMPLE_GUID: Guid = guid!("12345678-9abc-def0-1234-56789abcdef0");
/// ```
#[proc_macro]
pub fn guid(args: TokenStream) -> TokenStream {
    let (time_low, time_mid, time_high_and_version, clock_seq_and_variant, node) =
        match parse_guid(parse_macro_input!(args as LitStr)) {
            Ok(data) => data,
            Err(tokens) => return tokens.into(),
        };

    quote!({
        const g: ::uefi::Guid = ::uefi::Guid::from_values(
            #time_low,
            #time_mid,
            #time_high_and_version,
            #clock_seq_and_variant,
            #node,
        );
        g
    })
    .into()
}

fn parse_guid(guid_lit: LitStr) -> Result<(u32, u16, u16, u16, u64), TokenStream2> {
    let guid_str = guid_lit.value();

    // We expect a canonical GUID string, such as "12345678-9abc-def0-fedc-ba9876543210"
    if guid_str.len() != 36 {
        return Err(err!(
            guid_lit,
            "\"{}\" is not a canonical GUID string (expected 36 bytes, found {})",
            guid_str,
            guid_str.len()
        ));
    }
    let mut offset = 1; // 1 is for the starting quote
    let mut guid_hex_iter = guid_str.split('-');
    let mut next_guid_int = |len: usize| -> Result<u64, TokenStream2> {
        let guid_hex_component = guid_hex_iter.next().unwrap();

        // convert syn::LitStr to proc_macro2::Literal..
        let lit = match guid_lit.to_token_stream().into_iter().next().unwrap() {
            TokenTree::Literal(lit) => lit,
            _ => unreachable!(),
        };
        // ..so that we can call subspan and nightly users (us) will get the fancy span
        let span = lit
            .subspan(offset..offset + guid_hex_component.len())
            .unwrap_or_else(|| lit.span());

        if guid_hex_component.len() != len * 2 {
            return Err(err!(
                span,
                "GUID component \"{}\" is not a {}-bit hexadecimal string",
                guid_hex_component,
                len * 8
            ));
        }
        offset += guid_hex_component.len() + 1; // + 1 for the dash
        u64::from_str_radix(guid_hex_component, 16).map_err(|_| {
            err!(
                span,
                "GUID component \"{}\" is not a hexadecimal number",
                guid_hex_component
            )
        })
    };

    // The GUID string is composed of a 32-bit integer, three 16-bit ones, and a 48-bit one
    Ok((
        next_guid_int(4)? as u32,
        next_guid_int(2)? as u16,
        next_guid_int(2)? as u16,
        next_guid_int(2)? as u16,
        next_guid_int(6)?,
    ))
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

    // allow the entry function to be unsafe (by moving the keyword around so that it actually works)
    let unsafety = f.sig.unsafety.take();
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
    let signature_span = f.sig.span();

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
        #unsafety extern "efiapi" #f

    };
    result.into()
}

/// Builds a `CStr8` literal at compile time from a string literal.
///
/// This will throw a compile error if an invalid character is in the passed string.
///
/// # Example
/// ```
/// # use uefi_macros::cstr8;
/// assert_eq!(cstr8!("test").to_bytes_with_nul(), [116, 101, 115, 116, 0]);
/// ```
#[proc_macro]
pub fn cstr8(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: LitStr = parse_macro_input!(input);
    let input = input.value();
    match input
        .chars()
        .map(u8::try_from)
        .collect::<Result<Vec<u8>, _>>()
    {
        Ok(c) => {
            quote!(unsafe { ::uefi::CStr8::from_bytes_with_nul_unchecked(&[ #(#c),* , 0 ]) }).into()
        }
        Err(_) => syn::Error::new_spanned(input, "invalid character in string")
            .into_compile_error()
            .into(),
    }
}

/// Builds a `CStr16` literal at compile time from a string literal.
///
/// This will throw a compile error if an invalid character is in the passed string.
///
/// # Example
/// ```
/// # use uefi_macros::cstr16;
/// assert_eq!(cstr16!("test €").to_u16_slice_with_nul(), [116, 101, 115, 116, 32, 8364, 0]);
/// ```
#[proc_macro]
pub fn cstr16(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: LitStr = parse_macro_input!(input);
    let input = input.value();
    match input
        .chars()
        .map(|c| u16::try_from(c as u32))
        .collect::<Result<Vec<u16>, _>>()
    {
        Ok(c) => {
            quote!(unsafe { ::uefi::CStr16::from_u16_with_nul_unchecked(&[ #(#c),* , 0 ]) }).into()
        }
        Err(_) => syn::Error::new_spanned(input, "invalid character in string")
            .into_compile_error()
            .into(),
    }
}
