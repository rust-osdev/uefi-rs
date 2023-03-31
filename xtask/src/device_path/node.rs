use super::field::NodeField;
use super::group::DeviceType;
use crate::device_path::util::is_doc_attr;
use heck::ToShoutySnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Attribute, Fields, Ident, ItemStruct, LitInt, LitStr};

/// Device path node specification.
pub struct Node {
    /// Node doc attrs.
    pub docs: Vec<Attribute>,

    /// Device path node type.
    pub device_type: DeviceType,

    /// Device path node subtype.
    pub sub_type: Ident,

    /// Struct name (for both the packed and builder structs).
    pub struct_ident: Ident,

    /// Node fields.
    fields: Vec<NodeField>,

    /// Size (in bytes) of the node, including the four-byte
    /// header. Dynamically-sized fields are treated as zero bytes.
    static_size: usize,
}

impl Node {
    pub fn parse(struct_item: &ItemStruct, device_type: &DeviceType) -> Self {
        let struct_ident = struct_item.ident.clone();
        let sub_type = format!(
            "{}_{}",
            device_type.upper_name(),
            struct_item.ident.to_string().to_shouty_snake_case()
        );

        let mut out = Self {
            docs: Vec::new(),
            device_type: device_type.clone(),
            sub_type: Ident::new(&sub_type, Span::call_site()),
            struct_ident,
            fields: Vec::new(),
            static_size: 0,
        };

        for attr in &struct_item.attrs {
            if let Some(attr) = parse_node_attr(attr) {
                out.static_size = attr.static_size;
                if let Some(st) = attr.sub_type {
                    out.sub_type = Ident::new(&st, Span::call_site());
                }
            } else if is_doc_attr(attr) {
                out.docs.push(attr.clone());
            } else {
                panic!("unexpected attr: {}", quote!(#attr));
            }
        }

        if let Fields::Named(fields) = &struct_item.fields {
            for field in &fields.named {
                out.fields.push(NodeField::parse(field));
            }
        }

        // Check that the static_size attribute value matches adding up
        // all the non-DST field sizes. This serves as a quick check
        // that no fields were left out of the node specification.
        assert_eq!(out.calculate_static_size(), out.static_size);

        out
    }

    fn is_dst(&self) -> bool {
        if let Some(last) = self.fields.last() {
            last.is_slice()
        } else {
            false
        }
    }

    fn has_dst_group(&self) -> bool {
        self.fields.iter().filter(|field| field.is_slice()).count() > 1
    }

    /// Calculate the static size of the packed structure. This should
    /// give the same value as the `static_size` attribute.
    fn calculate_static_size(&self) -> usize {
        let header_size: usize = 4;
        let size = self.fields.iter().fold(header_size, |accum, field| {
            let field_size = field.packed_ty.size_in_bytes().unwrap_or(0);
            accum.checked_add(field_size).unwrap()
        });
        // Node lengths must fit in a u16.
        assert!(u16::try_from(size).is_ok());
        size
    }

    fn gen_packed_struct(&self) -> TokenStream {
        let struct_docs = &self.docs;
        let struct_ident = &self.struct_ident;

        let mut fields = vec![quote!(header: DevicePathHeader)];
        fields.extend(self.fields.iter().filter_map(|field| {
            // For a DST group, all the slice fields will be added as
            // one `data` slice below.
            if field.is_slice() && self.has_dst_group() {
                return None;
            }

            let field_name = &field.name;
            let field_ty = &field.packed_ty;
            Some(quote!(#field_name: #field_ty))
        }));

        // Combined `data` field for a DST group.
        if self.has_dst_group() {
            fields.push(quote!(data: [u8]));
        }

        // If the struct is a DST, derive the `ptr_meta::Pointee` trait.
        let derive_pointee = if self.is_dst() {
            quote!(#[derive(Pointee)])
        } else {
            quote!()
        };

        // For packed structs, we do not need the #[derive(Debug)] as we
        // generate an implementation.
        quote!(
            #(#struct_docs)*
            #[repr(C, packed)]
            #derive_pointee
            pub struct #struct_ident {
                #(pub(super) #fields),*
            }
        )
    }

    fn gen_packed_struct_impl(&self) -> TokenStream {
        let struct_ident = &self.struct_ident;

        let methods = self
            .fields
            .iter()
            .filter_map(NodeField::gen_packed_struct_get_method);

        quote!(
            impl #struct_ident {
                #(#methods)*
            }
        )
    }

    /// Generate a `fmt::Debug` impl for the packed struct.
    fn gen_packed_struct_debug_impl(&self) -> TokenStream {
        let struct_ident = &self.struct_ident;
        let struct_name = struct_ident.to_string();

        let mut field_calls: Vec<_> = self
            .fields
            .iter()
            .filter_map(|field| {
                let field_ident = &field.name;
                let field_name = field_ident.to_string();
                let field_val = quote!(self.#field_ident);
                let slice_elem_ty = field.slice_elem_ty();

                let dbg_val = if field.is_slice() {
                    if self.has_dst_group() {
                        return None;
                    }

                    // It's not trivial to nicely format the DST data since
                    // the slice might be unaligned. Treat it as a byte
                    // slice instead.
                    quote!({
                        let ptr = addr_of!(#field_val);
                        let (ptr, len) = PtrExt::to_raw_parts(ptr);
                        let byte_len = size_of::<#slice_elem_ty>() * len;
                        unsafe { &slice::from_raw_parts(ptr.cast::<u8>(), byte_len) }
                    })
                } else {
                    // Wrap in `{...}` to make a copy of the (potentially
                    // unaligned) data.
                    quote!(&{ #field_val })
                };

                Some(quote!(field(#field_name, #dbg_val)))
            })
            .collect();

        if self.has_dst_group() {
            field_calls.push(quote!(field("data", &&self.data)));
        }

        quote!(
            impl fmt::Debug for #struct_ident {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.debug_struct(#struct_name)
                        #(.#field_calls)*
                        .finish()
                }
            }
        )
    }

    pub fn gen_packed_struct_try_from_impl(&self) -> TokenStream {
        let struct_ident = &self.struct_ident;

        let try_from_body = if self.is_dst() {
            let slice_elem_ty = &self.fields.last().unwrap().slice_elem_ty().unwrap();

            let struct_ident = &self.struct_ident;
            let static_size = self.static_size;

            quote!(
                let static_size = #static_size;
                let dst_size = size_of_val(node).checked_sub(static_size).ok_or(
                    NodeConversionError::InvalidLength)?;
                let elem_size = size_of::<#slice_elem_ty>();
                if dst_size % elem_size != 0 {
                    return Err(NodeConversionError::InvalidLength);
                }
                let node: *const DevicePathNode = node;
                let node: *const #struct_ident = ptr_meta::from_raw_parts(node.cast(), dst_size / elem_size);
            )
        } else {
            quote!(
                if size_of_val(node) != size_of::<#struct_ident>() {
                    return Err(NodeConversionError::InvalidLength);
                }

                let node: *const DevicePathNode = node;
                let node: *const #struct_ident = node.cast();
            )
        };

        quote!(
            impl TryFrom<&DevicePathNode> for &#struct_ident {
                type Error = NodeConversionError;

                fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
                    #try_from_body

                    // Safety: the node fields have all been verified to
                    // be safe for arbitrary byte patterns (see
                    // `BaseType::new`), and the overall node size has
                    // been verified above, so this conversion is safe.
                    Ok(unsafe { &*node })
                }
            }
        )
    }

    pub fn gen_packed_code(&self) -> TokenStream {
        let s = self.gen_packed_struct();
        let s_impl = self.gen_packed_struct_impl();
        let dbg_impl = self.gen_packed_struct_debug_impl();
        let try_impl = self.gen_packed_struct_try_from_impl();
        quote!(
            #s
            #s_impl
            #dbg_impl
            #try_impl
        )
    }

    pub fn gen_builder_code(&self) -> TokenStream {
        let b = self.gen_builder();
        let b_impl = self.gen_builder_impl();
        quote!(
            #b
            #b_impl
        )
    }

    fn builder_lifetime(&self) -> Option<TokenStream> {
        if self.is_dst() {
            Some(quote!(<'a>))
        } else {
            None
        }
    }

    fn gen_builder(&self) -> TokenStream {
        let struct_ident = &self.struct_ident;
        let struct_docs = &self.docs;

        if self.fields.is_empty() {
            return quote!(
                #(#struct_docs)*
                #[derive(Debug)]
                pub struct #struct_ident;
            );
        }

        let fields = self.fields.iter().filter_map(|field| {
            // Skip hidden fields in the builder; these will be
            // initialized to zero automatically.
            if field.is_hidden() {
                return None;
            }

            let field_docs = &field.docs;
            let field_name = &field.name;
            field.build_type().map(|build_type| {
                Some(quote!(
                        #(#field_docs)*
                        pub #field_name: #build_type
                ))
            })
        });

        let struct_lifetime = self.builder_lifetime();
        quote!(
            #(#struct_docs)*
            #[derive(Debug)]
            pub struct #struct_ident #struct_lifetime {
                #(#fields),*
            }
        )
    }

    fn gen_builder_size_in_bytes_method(&self) -> TokenStream {
        let static_size = self.static_size;
        let dynamic_size = if self.is_dst() {
            let size_terms = self
                .fields
                .iter()
                .filter_map(NodeField::gen_builder_dynamic_size);
            Some(quote!(#(#size_terms)+*))
        } else {
            None
        };

        // Quoted code to calculate the total size of the packed
        // structure.
        let size = if self.is_dst() {
            quote!(#static_size + #dynamic_size)
        } else {
            quote!(#static_size)
        };

        quote!(
            fn size_in_bytes(&self) -> Result<u16, BuildError> {
                let size = #size;
                u16::try_from(size).map_err(|_| BuildError::NodeTooBig)
            }
        )
    }

    fn gen_builder_write_data_method(&self) -> TokenStream {
        let device_type = &self.device_type.const_ident();
        let sub_type = &self.sub_type;

        let mut copy_stmts = Vec::new();

        if self.has_dst_group() {
            copy_stmts.push(quote!(
                let mut dst_group_offset = 0;
            ));
        }

        // Offset of the current field. Start at 4 to skip past the header bytes.
        let mut field_offset = 4usize;

        // Quoted code to initialize each field in the packed struct.
        copy_stmts.extend(self.fields.iter().enumerate().map(|(index, field)| {
            let field_name = &field.name;
            let packed_ty = &field.packed_ty;
            let custom_build_func = Ident::new(&format!("build_{}", field.name), Span::call_site());

            let cs = if field.is_slice() {
                if self.has_dst_group() {
                    let write_field = field.gen_builder_write_slice(
                        quote!(out_ptr.add(#field_offset + dst_group_offset)),
                    );

                    // Skip updating the dst offset on the last
                    // element.
                    if index == self.fields.len() - 1 {
                        write_field
                    } else {
                        quote!(
                            #write_field;
                            dst_group_offset += size_of_val(self.#field_name);
                        )
                    }
                } else if field.attr.custom_build_impl {
                    quote!(self.#custom_build_func(
                        &mut out[#field_offset..]
                    ))
                } else {
                    field.gen_builder_write_slice(quote!(out_ptr.add(#field_offset)))
                }
            } else if field.is_hidden() {
                // Initialize hidden fields with zeroes.
                quote!(
                    out_ptr
                        .add(#field_offset)
                        .write_bytes(0, size_of::<#packed_ty>());)
            } else {
                let val = if field.attr.custom_build_impl {
                    quote!(self.#custom_build_func())
                } else {
                    quote!(self.#field_name)
                };
                quote!(
                    out_ptr
                        .add(#field_offset)
                        .cast::<#packed_ty>()
                        .write_unaligned(#val);
                )
            };

            // Update the field offset. Slices always come at the end of
            // the struct, so no need to update the offset for that case.
            if let Some(size) = packed_ty.size_in_bytes() {
                field_offset += size;
            }

            cs
        }));

        quote!(
            fn write_data(&self, out: &mut [MaybeUninit<u8>]) {
                let size = usize::from(self.size_in_bytes().unwrap());
                // Ensure that the caller upholds the contract that
                // the length of `out` matches the node's size.
                assert_eq!(size, out.len());

                let out_ptr: *mut u8 = maybe_uninit_slice_as_mut_ptr(out);
                unsafe {
                    out_ptr.cast::<DevicePathHeader>().write_unaligned(DevicePathHeader {
                        device_type: DeviceType::#device_type,
                        sub_type: DeviceSubType::#sub_type,
                        length: u16::try_from(size).unwrap(),
                    });
                    #(#copy_stmts)*
                }
            }
        )
    }

    fn gen_builder_impl(&self) -> TokenStream {
        let struct_ident = &self.struct_ident;
        let lifetime = self.builder_lifetime();

        let size_in_bytes_method = self.gen_builder_size_in_bytes_method();
        let write_data_method = self.gen_builder_write_data_method();

        quote!(
            unsafe impl #lifetime BuildNode for #struct_ident #lifetime {
                #size_in_bytes_method

                #write_data_method
            }
        )
    }
}

struct NodeAttr {
    static_size: usize,
    sub_type: Option<String>,
}

/// Parse a `node` attribute. Returns `None` for any other attribute, or
/// if the contents don't match the expected format.
fn parse_node_attr(attr: &Attribute) -> Option<NodeAttr> {
    if !attr.path().is_ident("node") {
        return None;
    }

    let mut static_size = None;
    let mut sub_type = None;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("static_size") {
            let value = meta.value()?;
            let lit: LitInt = value.parse()?;
            let lit = lit.base10_parse()?;
            static_size = Some(lit);
            Ok(())
        } else if meta.path.is_ident("sub_type") {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            sub_type = Some(lit.value());
            Ok(())
        } else {
            Err(meta.error("invalid struct node attribute"))
        }
    })
    .ok()?;

    Some(NodeAttr {
        static_size: static_size?,
        sub_type,
    })
}

/// Returns `true` if the attribute is a valid `node` attribute, false
/// otherwise.
pub fn is_node_attr(attr: &Attribute) -> bool {
    parse_node_attr(attr).is_some()
}
