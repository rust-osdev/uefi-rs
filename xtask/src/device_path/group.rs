use super::node::{is_node_attr, Node};
use heck::ToUpperCamelCase;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Attribute, Ident, Item, ItemMod, ItemStruct};

#[derive(Clone)]
pub struct DeviceType(Ident);

impl DeviceType {
    pub fn module_ident(&self) -> &Ident {
        &self.0
    }

    pub fn const_ident(&self) -> Ident {
        Ident::new(&self.upper_name(), Span::call_site())
    }

    pub fn upper_name(&self) -> String {
        self.0.to_string().to_uppercase()
    }

    pub fn camel_name(&self) -> String {
        self.0.to_string().to_upper_camel_case()
    }
}

pub struct NodeGroup {
    device_type: DeviceType,

    /// Device path node specifications.
    nodes: Vec<Node>,

    /// Non-node items in the group that are passed through unchanged to
    /// the packed module.
    friends: Vec<Item>,

    /// Non-node items in the group that are passed through unchanged to
    /// the build module.
    build_friends: Vec<Item>,
}

impl NodeGroup {
    pub fn parse(module: &ItemMod) -> Self {
        let items = &module.content.as_ref().unwrap().1;

        let device_type = DeviceType(module.ident.clone());

        let mut group = Self {
            device_type,
            nodes: Vec::new(),
            friends: Vec::new(),
            build_friends: Vec::new(),
        };

        for item in items {
            if let Some(struct_item) = get_node_struct(item) {
                group
                    .nodes
                    .push(Node::parse(struct_item, &group.device_type));
            } else {
                let mut item = item.clone();
                if has_build_attr(&item) {
                    remove_build_attr(&mut item);
                    group.build_friends.push(item);
                } else {
                    group.friends.push(item);
                }
            }
        }

        group
    }

    pub fn gen_packed_module(&self) -> TokenStream {
        let module_ident = &self.device_type.module_ident();
        let nodes = self.nodes.iter().map(Node::gen_packed_code);
        let friends = &self.friends;
        let doc = format!(
            " Device path nodes for [`DeviceType::{}`].",
            self.device_type.const_ident()
        );

        quote!(
            #[doc = #doc]
            pub mod #module_ident {
                use super::*;

                #(#nodes)*
                #(#friends)*
            }
        )
    }

    pub fn gen_builder_module(&self) -> TokenStream {
        let module_ident = &self.device_type.module_ident();
        let nodes = self.nodes.iter().map(Node::gen_builder_code);
        let friends = &self.build_friends;
        let doc = format!(
            " Device path build nodes for [`DeviceType::{}`].",
            self.device_type.const_ident()
        );

        quote!(
            #[doc = #doc]
            pub mod #module_ident {
                use super::*;

                #(#nodes)*
                #(#friends)*
            }
        )
    }

    /// Generate the `DevicePathNodeEnum` enum.
    pub fn gen_node_enum(groups: &[NodeGroup]) -> TokenStream {
        let variant_name = |module: &NodeGroup, node: &Node| {
            Ident::new(
                &format!("{}{}", module.device_type.camel_name(), node.struct_ident),
                Span::call_site(),
            )
        };

        let variants = groups.iter().flat_map(|module| {
            module.nodes.iter().map(|node| {
                let module_name = &module.device_type.module_ident();
                let struct_ident = &node.struct_ident;
                let variant_name = variant_name(module, node);
                let docs = &node.docs;
                quote!(
                    #(#docs)*
                    #variant_name(&'a #module_name::#struct_ident)
                )
            })
        });

        let try_from_arms = groups.iter().flat_map(|module| {
            module.nodes.iter().map(|node| {
                let variant_name = variant_name(module, node);

                let device_type = &node.device_type.const_ident();
                let sub_type = &node.sub_type;

                quote!(
                    (DeviceType::#device_type, DeviceSubType::#sub_type) =>
                        Self::#variant_name(node.try_into()?)
                )
            })
        });

        quote!(
            /// Enum of references to all the different device path node
            /// types. Return type of [`DevicePathNode::as_enum`].
            #[derive(Debug)]
            pub enum DevicePathNodeEnum<'a> {
                #(#variants),*
            }

            impl<'a> TryFrom<&DevicePathNode> for DevicePathNodeEnum<'a> {
                type Error = NodeConversionError;

                fn try_from(node: &DevicePathNode) -> Result<Self, Self::Error> {
                    Ok(match node.full_type() {
                        #(#try_from_arms),*,
                        _ => return Err(NodeConversionError::UnsupportedType),
                    })
                }
            }
        )
    }
}

fn is_build_attr(attr: &Attribute) -> bool {
    attr.path().is_ident("build")
}

fn has_build_attr(item: &Item) -> bool {
    let attrs = match item {
        Item::Impl(item) => &item.attrs,
        Item::Struct(item) => &item.attrs,
        _ => return false,
    };

    attrs.iter().any(is_build_attr)
}

fn remove_build_attr(item: &mut Item) {
    let attrs = match item {
        Item::Impl(item) => &mut item.attrs,
        Item::Struct(item) => &mut item.attrs,
        _ => return,
    };

    attrs.retain(|attr| !is_build_attr(attr));
}

/// Check if the item is a struct with the `#[node(...)]`
/// attribute. Return the item as an `&ItemStruct` if so, otherwise
/// return None.
fn get_node_struct(item: &Item) -> Option<&ItemStruct> {
    if let Item::Struct(item) = item {
        if item.attrs.iter().any(is_node_attr) {
            return Some(item);
        }
    }
    None
}
