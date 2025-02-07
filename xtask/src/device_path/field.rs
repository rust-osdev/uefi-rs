// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::device_path::util::is_doc_attr;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Attribute, Expr, ExprLit, Field, Ident, Lit, Path, Type, TypeArray};

/// A fixed-size non-array type.
///
/// All base types must support arbitrary byte patterns. For example,
/// `bool` would not be safe since only 0 and 1 are valid `bool` values;
/// `u8` should be used instead.
///
/// This requirement is needed to make the node conversion functions
/// safe without needing to inspect each field.
///
/// To add a new base type, verify that it meets the above requirement,
/// then add it to the list in `BaseType::new` along with its size.
#[derive(Clone)]
pub struct BaseType {
    path: Path,
    size_in_bytes: usize,
}

impl BaseType {
    fn new(ty: &Type) -> Self {
        if let Type::Path(ty) = ty {
            let path = ty.path.clone();
            let path_str = path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            let size_in_bytes = match path_str.as_str() {
                "u8" => 1,
                "u16" => 2,
                "u32" => 4,
                "u64" => 8,

                "Guid" => 16,
                "IpAddress" => 16,
                "MemoryType" => 4,

                "device_path::hardware::BmcInterfaceType" => 1,

                "device_path::messaging::BluetoothLeAddressType" => 1,
                "device_path::messaging::DnsAddressType" => 1,
                "device_path::messaging::InfinibandResourceFlags" => 4,
                "device_path::messaging::Ipv4AddressOrigin" => 1,
                "device_path::messaging::Ipv6AddressOrigin" => 1,
                "device_path::messaging::IscsiLoginOptions" => 2,
                "device_path::messaging::IscsiProtocol" => 2,
                "device_path::messaging::MasterSlave" => 1,
                "device_path::messaging::Parity" => 1,
                "device_path::messaging::PrimarySecondary" => 1,
                "device_path::messaging::RestServiceAccessMode" => 1,
                "device_path::messaging::RestServiceType" => 1,
                "device_path::messaging::StopBits" => 1,

                "device_path::media::PartitionFormat" => 1,
                "device_path::media::RamDiskType" => 16,
                "device_path::media::SignatureType" => 1,

                _ => panic!("unsupported base type: {path_str}"),
            };
            Self {
                path,
                size_in_bytes,
            }
        } else {
            panic!("invalid base type: {}", quote!(#ty));
        }
    }

    pub fn is_u8(&self) -> bool {
        if let Some(ident) = self.path.get_ident() {
            ident == "u8"
        } else {
            false
        }
    }
}

impl ToTokens for BaseType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.path.to_tokens(tokens);
    }
}

/// Storage type for a field in the packed struct.
#[derive(Clone)]
pub enum PackedType {
    /// A fixed-size non-array type.
    Base(BaseType),

    /// A fixed-size array containing `BaseType`s.
    Array(BaseType, usize),

    /// A dynamically-sized slice containing `BaseType`.
    Slice(BaseType),
}

impl PackedType {
    fn new(ty: &Type) -> Self {
        match ty {
            Type::Slice(slice) => Self::Slice(BaseType::new(&slice.elem)),
            Type::Array(TypeArray {
                elem,
                len:
                    Expr::Lit(ExprLit {
                        lit: Lit::Int(len), ..
                    }),
                ..
            }) => {
                let len = len.base10_parse::<usize>().unwrap();
                Self::Array(BaseType::new(elem), len)
            }
            _ => Self::Base(BaseType::new(ty)),
        }
    }

    pub fn size_in_bytes(&self) -> Option<usize> {
        match self {
            Self::Base(base) => Some(base.size_in_bytes),
            Self::Array(base, len) => Some(base.size_in_bytes * len),
            Self::Slice(_) => None,
        }
    }
}

impl ToTokens for PackedType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Base(base) => base.to_tokens(tokens),
            Self::Array(base, len) => tokens.append_all(quote!([#base; #len])),
            Self::Slice(base) => tokens.append_all(quote!([#base])),
        }
    }
}

enum BuildType {
    None,
    Packed,
    Custom(Type),
}

#[derive(PartialEq)]
pub enum GetFunc {
    /// No getter will be generated.
    None,

    /// Autogenerate the getter.
    Auto,

    /// Autogenerate the getter, but call a custom function to get the
    /// return value.
    Custom,
}

pub struct NodeField {
    pub docs: Vec<Attribute>,
    pub name: Ident,

    /// The type used in the packed node structure.
    pub packed_ty: PackedType,

    pub attr: FieldNodeAttr,
}

impl NodeField {
    pub fn parse(field: &Field) -> Self {
        let mut out = Self {
            docs: Vec::new(),
            name: field.ident.clone().unwrap(),
            packed_ty: PackedType::new(&field.ty),
            attr: FieldNodeAttr::default(),
        };

        for attr in &field.attrs {
            if let Some(attr) = FieldNodeAttr::from_attr(attr) {
                out.attr = attr;
            } else if is_doc_attr(attr) {
                out.docs.push(attr.clone());
            } else {
                panic!("unexpected attr: {}", quote!(#attr));
            }
        }

        out
    }

    pub fn is_slice(&self) -> bool {
        self.slice_elem_ty().is_some()
    }

    pub fn slice_elem_ty(&self) -> Option<&BaseType> {
        if let PackedType::Slice(slice) = &self.packed_ty {
            Some(slice)
        } else {
            None
        }
    }

    /// Get the field type for a uefi-raw node.
    ///
    /// This is mostly the same as `packed_ty`, except that DST slices are
    /// converted to zero-length arrays.
    pub fn raw_ty(&self) -> PackedType {
        if let PackedType::Slice(ty) = &self.packed_ty {
            PackedType::Array(ty.clone(), 0)
        } else {
            self.packed_ty.clone()
        }
    }

    /// Whether the field is internal-only, e.g. a reserved field. No
    /// accessor will be generated for the field and the builder will
    /// initialize it with zeros.
    pub fn is_hidden(&self) -> bool {
        self.name.to_string().starts_with('_')
    }

    pub fn build_type(&self) -> Option<TokenStream> {
        match &self.attr.build_type {
            BuildType::None => None,
            BuildType::Custom(build_type) => Some(quote!(#build_type)),
            BuildType::Packed => {
                let packed_ty = &self.packed_ty;
                if self.is_slice() {
                    Some(quote!(&'a #packed_ty))
                } else {
                    Some(quote!(#packed_ty))
                }
            }
        }
    }

    /// Generate the packed struct method to get this field.
    ///
    /// The generated method will return a copy for non-DST fields. For
    /// DSTs it will return `&[u8]` for `[u8]` slices, and an
    /// `UnalignedSlice` for all other slice types.
    pub fn gen_packed_struct_get_method(&self) -> Option<TokenStream> {
        if self.is_hidden() || self.attr.get_func == GetFunc::None {
            return None;
        }

        let field_name = &self.name;
        let field_ty = &self.packed_ty;
        let field_docs = &self.docs;

        let ret_type;
        let mut ret_val;

        if let PackedType::Slice(slice_elem) = &self.packed_ty {
            if slice_elem.is_u8() {
                // Special handling for [u8]: there are no alignment
                // concerns so we can return a reference.
                ret_type = quote!(& #field_ty);
                ret_val = quote!(&self.#field_name);
            } else {
                // In the general case we can't safely return a
                // reference to the slice since it might be
                // unaligned, so use `UnalignedSlice`.
                ret_type = quote!(UnalignedSlice<#slice_elem>);
                ret_val = quote!(
                    let ptr: *const [#slice_elem] = addr_of!(self.#field_name);
                    let (ptr, len): (*const (), usize) = ptr_meta::to_raw_parts(ptr);
                    unsafe {
                        UnalignedSlice::new(ptr.cast::<#slice_elem>(), len)
                    }
                );
            }
        } else {
            ret_type = quote!(#field_ty);
            ret_val = quote!(self.#field_name);
        }

        if self.attr.get_func == GetFunc::Custom {
            let get_func = Ident::new(&format!("get_{}", self.name), Span::call_site());
            ret_val = quote!(self.#get_func());
        }

        Some(quote!(
            #(#field_docs)*
            #[must_use]
            pub fn #field_name(&self) -> #ret_type {
                #ret_val
            }
        ))
    }

    /// Generate code to calculate the size of DST fields. Returns
    /// `None` for non-DST fields.
    pub fn gen_builder_dynamic_size(&self) -> Option<TokenStream> {
        if self.attr.custom_build_size_impl {
            let size_func_name = format!("build_size_{}", self.name);
            let size_func = Ident::new(&size_func_name, Span::call_site());
            Some(quote!(self.#size_func()))
        } else if self.is_slice() {
            let field_name = &self.name;
            Some(quote!(size_of_val(self.#field_name)))
        } else {
            None
        }
    }

    /// Generate the code for writing a slice to the packed output.
    pub fn gen_builder_write_slice(&self, out_ptr: TokenStream) -> TokenStream {
        assert!(self.is_slice());

        let field_name = &self.name;
        let size = self.gen_builder_dynamic_size();
        quote!(
            self.#field_name
                .as_ptr()
                .cast::<u8>()
                .copy_to_nonoverlapping(
                    #out_ptr,
                    #size);
        )
    }
}

/// Field customizations controlled by a `#[node(...)]` attr.
pub struct FieldNodeAttr {
    // What kind of packed getter to generate.
    pub get_func: GetFunc,

    /// The type used in the build node structure. `Packed` by default.
    build_type: BuildType,

    /// If true, the autogenerated build code calls a custom method
    /// named `build_<field>`. False by default.
    pub custom_build_impl: bool,

    /// If true, the autogenerated code to calculate the size of the
    /// field when building a node calls a custom method named
    /// `build_size_<field>`. False by default.
    pub custom_build_size_impl: bool,
}

impl Default for FieldNodeAttr {
    fn default() -> Self {
        Self {
            get_func: GetFunc::Auto,
            build_type: BuildType::Packed,
            custom_build_impl: false,
            custom_build_size_impl: false,
        }
    }
}

impl FieldNodeAttr {
    /// Parse a field `node` attribute as described in the
    /// readme. Returns `None` if the attribute does not exactly match
    /// the expected format.
    fn from_attr(attr: &Attribute) -> Option<Self> {
        if !attr.path().is_ident("node") {
            return None;
        }

        let mut out = Self::default();

        attr.parse_nested_meta(|meta| {
            let path = &meta.path;
            if path.is_ident("no_get_func") {
                out.get_func = GetFunc::None;
            } else if path.is_ident("custom_get_impl") {
                out.get_func = GetFunc::Custom;
            } else if path.is_ident("custom_build_impl") {
                out.custom_build_impl = true;
            } else if path.is_ident("custom_build_size_impl") {
                out.custom_build_size_impl = true;
            } else if path.is_ident("build_type") {
                let value = meta.value()?;
                let lit: Lit = value.parse()?;

                match lit {
                    Lit::Str(s) => {
                        out.build_type = BuildType::Custom(syn::parse_str(&s.value())?);
                    }
                    Lit::Bool(b) if !b.value() => {
                        out.build_type = BuildType::None;
                    }
                    _ => {
                        return Err(meta.error("invalid build_type"));
                    }
                }
            } else {
                return Err(meta.error("invalid field node attribute"));
            }
            Ok(())
        })
        .ok()?;

        Some(out)
    }
}
