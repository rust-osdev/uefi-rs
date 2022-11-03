mod field;
mod group;
mod node;
mod util;

use crate::opt::GenCodeOpt;
use anyhow::{bail, Result};
use fs_err as fs;
use group::NodeGroup;
use quote::quote;
use syn::{File, Item};
use util::rustfmt_string;

const INPUT_PATH: &str = "xtask/src/device_path/spec.rs";
const OUTPUT_PATH: &str = "src/proto/device_path/device_path_gen.rs";

fn gen_code_as_string(groups: &[NodeGroup]) -> Result<String> {
    let packed_modules = groups.iter().map(NodeGroup::gen_packed_module);
    let node_enum = NodeGroup::gen_node_enum(groups);
    let build_modules = groups.iter().map(NodeGroup::gen_builder_module);

    let code = quote!(
        use bitflags::bitflags;
        use crate::data_types::UnalignedSlice;
        use crate::{guid, Guid};
        use crate::proto::device_path::{
            DevicePathHeader, DevicePathNode, DeviceSubType, DeviceType,
            NodeConversionError,
        };
        use crate::proto::network::IpAddress;
        use crate::table::boot::MemoryType;
        use core::mem::{size_of, size_of_val};
        use core::ptr::{self, addr_of};
        use core::{fmt, slice};

        #(#packed_modules)*

        #node_enum

        /// Build device paths from their component nodes.
        pub mod build {
            use super::*;

            use core::mem::{MaybeUninit, size_of_val};
            use crate::CStr16;
            use crate::proto::device_path::build::{BuildError, BuildNode};
            use crate::proto::device_path::{DeviceSubType, DeviceType};

            #(#build_modules)*
        }
    );

    // Insert some blank lines to make the output a bit more readable,
    // otherwise everything is entirely squished together. `rustfmt`
    // doesn't currently handle inserting blank lines very well, even
    // with the unstable options.
    let code = code.to_string().replace('}', "}\n\n");

    let output = format!(
        "
// DO NOT EDIT
//
// This file was automatically generated with:
// `cargo xtask gen-code`
//
// See //xtask/src/device_path/README.md for more details.

{code}"
    );

    let formatted = rustfmt_string(output)?;

    Ok(formatted)
}

fn parse_spec(spec_str: &str) -> Vec<NodeGroup> {
    let ast: File = syn::parse_str(spec_str).unwrap();

    ast.items
        .iter()
        .map(|item| {
            if let Item::Mod(module) = item {
                NodeGroup::parse(module)
            } else {
                panic!("unexpected item")
            }
        })
        .collect()
}

pub fn gen_code(opt: &GenCodeOpt) -> Result<()> {
    let spec_str = include_str!("spec.rs");

    let groups = parse_spec(spec_str);
    let output_string = gen_code_as_string(&groups)?;

    if opt.check {
        // Implementation note: we don't use `rustfmt --check` because
        // it always exits zero when reading from stdin:
        // https://github.com/rust-lang/rustfmt/issues/5376

        if output_string != fs::read_to_string(OUTPUT_PATH)? {
            bail!("generated code is stale");
        }

        // Also check the input file's formatting.
        if spec_str != rustfmt_string(spec_str.to_owned())? {
            bail!("spec.rs needs formatting");
        }
    } else {
        fs::write(OUTPUT_PATH, output_string)?;

        // Also format the input file. It's valid rust, but not included
        // via `mod` anywhere, so the usual `cargo fmt --all` doesn't
        // update it.
        let input = rustfmt_string(fs::read_to_string(INPUT_PATH)?)?;
        fs::write(INPUT_PATH, input)?;
    }

    Ok(())
}
