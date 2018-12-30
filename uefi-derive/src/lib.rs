#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Lit, Meta};

// Custom derive for the Identify trait
#[proc_macro_derive(Identify, attributes(unsafe_guid))]
pub fn derive_identify_impl(item: TokenStream) -> TokenStream {
    // Parse the input using Syn
    let item = parse_macro_input!(item as DeriveInput);

    // Look for struct-wide #[unsafe_guid = "..."] attributes
    let guid_ident = Ident::new("unsafe_guid", Span::call_site().into());
    let guid_attrs = item
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident(guid_ident.clone()))
        .collect::<Vec<_>>();

    // There must be exactly one such attribute
    let guid_attr = match guid_attrs.len() {
        0 => panic!(
            "In order to derive Identify, the type's GUID must be specified. \
             You can set it using the #[unsafe_guid = \"...\"] attribute."
        ),
        1 => &guid_attrs[0],
        n => panic!(
            "Expected a single unsafe_guid attribute, found {} of them.",
            n
        ),
    };

    // The unsafe_guid attribute must use the MetaNameValue syntax with a string argument
    let guid_lit = match guid_attr.parse_meta() {
        Ok(Meta::NameValue(nv)) => nv.lit,
        _ => panic!("The unsafe_guid attribute is spelled #[unsafe_guid = \"...\"]"),
    };
    let guid_str = match guid_lit {
        Lit::Str(s) => s.value(),
        _ => panic!("The unsafe_guid attribute is spelled #[unsafe_guid = \"...\"]"),
    };

    // We expect a GUID in canonical form, such as "12345678-9abc-def0-fedc-ba9876543210"
    let mut guid_hex_iter = guid_str.split('-');
    let guid_component_count = guid_hex_iter.clone().count();
    if guid_component_count != 5 {
        panic!(
            "\"{}\" is not a canonical GUID (expected 5 hyphen-separated components, found {})",
            guid_str, guid_component_count
        );
    }
    let mut next_guid_int = |expected_num_bits: u32| -> u64 {
        let guid_hex_component = guid_hex_iter.next().unwrap();
        let guid_component = match u64::from_str_radix(guid_hex_component, 16) {
            Ok(number) => number,
            _ => panic!(
                "GUID component \"{}\" is not a hexadecimal number",
                guid_hex_component
            ),
        };
        if guid_component.leading_zeros() < 64 - expected_num_bits {
            panic!(
                "GUID component \"{}\" is not a {}-bit hexadecimal number",
                guid_hex_component, expected_num_bits
            );
        }
        guid_component
    };

    // These are, in order, a 32-bit interger, three 16-bit ones, and a 48-bit one
    let time_low = next_guid_int(32) as u32;
    let time_mid = next_guid_int(16) as u16;
    let time_high_and_version = next_guid_int(16) as u16;
    let clock_seq_and_variant = next_guid_int(16) as u16;
    let node_64 = next_guid_int(48);

    // Convert the node ID to an array of bytes to comply with Guid::from_values expectations
    let node = [
        (node_64 >> 40) as u8,
        ((node_64 >> 32) % 0x100) as u8,
        ((node_64 >> 24) % 0x100) as u8,
        ((node_64 >> 16) % 0x100) as u8,
        ((node_64 >> 8) % 0x100) as u8,
        (node_64 % 0x100) as u8,
    ];

    // At this point, we know everything we need to implement Identify
    let ident = item.ident.clone();
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let result = quote! {
        unsafe impl #impl_generics crate::Identify for #ident #ty_generics #where_clause {
            #[doc(hidden)]
            #[allow(clippy::unreadable_literal)]
            const GUID : crate::Guid = crate::Guid::from_values(
                #time_low,
                #time_mid,
                #time_high_and_version,
                #clock_seq_and_variant,
                [#(#node),*],
            );
        }
    };
    result.into()
}

// Custom derive for the Protocol trait
#[proc_macro_derive(Protocol)]
pub fn derive_protocol(item: TokenStream) -> TokenStream {
    // Parse the input using Syn
    let item = parse_macro_input!(item as DeriveInput);

    // Then implement Protocol
    let ident = item.ident.clone();
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let result = quote! {
        // Mark this as a Protocol implementation
        impl #impl_generics crate::proto::Protocol for #ident #ty_generics #where_clause {}

        // Most UEFI functions expect to be called on the bootstrap processor.
        impl #impl_generics !Send for #ident #ty_generics #where_clause {}

        // Most UEFI functions do not support multithreaded access.
        impl #impl_generics !Sync for #ident #ty_generics #where_clause {}
    };
    result.into()
}
