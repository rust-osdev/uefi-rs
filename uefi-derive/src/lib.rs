#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Lit, Meta, NestedMeta};

// Custom derive for the Identify trait
#[proc_macro_derive(Identify, attributes(unsafe_guid))]
pub fn derive_identify_impl(item: TokenStream) -> TokenStream {
    // Parse the input using Syn
    let item = parse_macro_input!(item as DeriveInput);

    // Look for struct-wide #[unsafe_guid(...)] attributes
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
             You can set it using the #[unsafe_guid(...)] attribute."
        ),
        1 => &guid_attrs[0],
        n => panic!(
            "Expected a single unsafe_guid attribute, found {} of them.",
            n
        ),
    };

    // The unsafe_guid attribute must use the MetaList syntax
    let guid_list = match guid_attr.parse_meta() {
        Ok(Meta::List(list)) => list,
        _ => panic!("The unsafe_guid attribute is spelled #[unsafe_guid(...)]"),
    };

    // The unsafe_guid attribute takes 5 integer inputs of variable width
    let guid_elems = guid_list.nested;
    if guid_elems.len() != 5 {
        panic!(
            "The unsafe_guid attribute takes 5 parameters, but {} were provided.",
            guid_elems.len()
        );
    }
    let mut guid_elems_iter = guid_elems.iter();
    let mut next_guid_int = |num_bits: u32| -> u64 {
        let val_64 = match guid_elems_iter.next().unwrap() {
            NestedMeta::Literal(Lit::Int(x)) => x.value(),
            _ => panic!("Inputs to unsafe_guid must be integer literals."),
        };
        if val_64.leading_zeros() < 64 - num_bits {
            panic!(
                "The unsafe_guid input {:x} is not a valid {}-bit integer.",
                val_64, num_bits
            );
        }
        val_64
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
