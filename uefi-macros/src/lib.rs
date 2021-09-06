#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::Span;
use quote::{quote, TokenStreamExt};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, DeriveInput, Generics, Ident, ItemFn, ItemType, LitStr};

/// Parses a type definition, extracts its identifier and generic parameters
struct TypeDefinition {
    ident: Ident,
    generics: Generics,
}

impl Parse for TypeDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(d) = DeriveInput::parse(input) {
            Ok(Self {
                ident: d.ident,
                generics: d.generics,
            })
        } else if let Ok(t) = ItemType::parse(input) {
            Ok(Self {
                ident: t.ident,
                generics: t.generics,
            })
        } else {
            Err(input.error("Input is not an alias, enum, struct or union definition"))
        }
    }
}

/// `unsafe_guid` attribute macro, implements the `Identify` trait for any type
/// (mostly works like a custom derive, but also supports type aliases)
#[proc_macro_attribute]
pub fn unsafe_guid(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the arguments and input using Syn
    let guid_str = parse_macro_input!(args as LitStr).value();
    let mut result: proc_macro2::TokenStream = input.clone().into();
    let type_definition = parse_macro_input!(input as TypeDefinition);

    // We expect a canonical GUID string, such as "12345678-9abc-def0-fedc-ba9876543210"
    if guid_str.len() != 36 {
        panic!(
            "\"{}\" is not a canonical GUID string (expected 36 bytes, found {})",
            guid_str,
            guid_str.len()
        );
    }
    let mut guid_hex_iter = guid_str.split('-');
    let mut next_guid_int = |expected_num_bits: usize| -> u64 {
        let guid_hex_component = guid_hex_iter.next().unwrap();
        if guid_hex_component.len() != expected_num_bits / 4 {
            panic!(
                "GUID component \"{}\" is not a {}-bit hexadecimal string",
                guid_hex_component, expected_num_bits
            );
        }
        match u64::from_str_radix(guid_hex_component, 16) {
            Ok(number) => number,
            _ => panic!(
                "GUID component \"{}\" is not a hexadecimal number",
                guid_hex_component
            ),
        }
    };

    // The GUID string is composed of a 32-bit integer, three 16-bit ones, and a 48-bit one
    let time_low = next_guid_int(32) as u32;
    let time_mid = next_guid_int(16) as u16;
    let time_high_and_version = next_guid_int(16) as u16;
    let clock_seq_and_variant = next_guid_int(16) as u16;
    let node = next_guid_int(48);

    // At this point, we know everything we need to implement Identify
    let ident = type_definition.ident.clone();
    let (impl_generics, ty_generics, where_clause) = type_definition.generics.split_for_impl();
    result.append_all(quote! {
        unsafe impl #impl_generics ::uefi::Identify for #ident #ty_generics #where_clause {
            #[doc(hidden)]
            #[allow(clippy::unreadable_literal)]
            const GUID: ::uefi::Guid = ::uefi::Guid::from_values(
                #time_low,
                #time_mid,
                #time_high_and_version,
                #clock_seq_and_variant,
                #node,
            );
        }
    });
    result.into()
}

/// Custom derive for the `Protocol` trait
#[proc_macro_derive(Protocol)]
pub fn derive_protocol(item: TokenStream) -> TokenStream {
    // Parse the input using Syn
    let item = parse_macro_input!(item as DeriveInput);

    // Then implement Protocol
    let ident = item.ident.clone();
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let result = quote! {
        // Mark this as a `Protocol` implementation
        impl #impl_generics ::uefi::proto::Protocol for #ident #ty_generics #where_clause {}

        // Most UEFI functions expect to be called on the bootstrap processor.
        impl #impl_generics !Send for #ident #ty_generics #where_clause {}

        // Most UEFI functions do not support multithreaded access.
        impl #impl_generics !Sync for #ident #ty_generics #where_clause {}
    };
    result.into()
}

/// Custom attribute for a UEFI executable entrypoint
#[proc_macro_attribute]
pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    // This code is inspired by the approach in this embedded Rust crate:
    // https://github.com/rust-embedded/cortex-m-rt/blob/965bf1e3291571e7e3b34834864117dc020fb391/macros/src/lib.rs#L85

    if !args.is_empty() {
        panic!("This attribute accepts no arguments");
    }

    let mut f = parse_macro_input!(input as ItemFn);

    // force the exported symbol to be 'efi_main'
    f.sig.ident = Ident::new("efi_main", Span::call_site());

    let result = quote! {
        static _UEFI_ENTRY_POINT_TYPE_CHECK: extern "efiapi" fn(uefi::Handle, uefi::table::SystemTable<uefi::table::Boot>) -> uefi::Status = efi_main;
        #[no_mangle]
        pub extern "efiapi" #f
    };
    result.into()
}
