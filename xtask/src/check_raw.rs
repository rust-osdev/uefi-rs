// SPDX-License-Identifier: MIT OR Apache-2.0

//! Validate various properties of the code in the `uefi-raw` package.
//!
//! For example, this checks that no Rust enums are used, that structs have an
//! appropriate repr for FFI, that raw pointers are used instead of references,
//! and many other things.

use anyhow::Result;
use fs_err as fs;
use proc_macro2::TokenStream;
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};
use std::process;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{
    Abi, Attribute, Field, Fields, FieldsNamed, FieldsUnnamed, File, Item, ItemConst, ItemMacro,
    ItemStruct, ItemType, ItemUnion, LitInt, ReturnType, Type, TypeArray, TypeBareFn, TypePtr,
    Visibility, parenthesized,
};
use walkdir::WalkDir;

/// Type of an `Item`.
#[derive(Debug, Eq, PartialEq)]
enum ItemKind {
    Enum,
    Other,
}

impl From<&Item> for ItemKind {
    fn from(item: &Item) -> Self {
        match item {
            Item::Enum(_) => Self::Enum,
            _ => Self::Other,
        }
    }
}

/// All possible validation error kinds.
#[derive(Debug, Eq, PartialEq)]
enum ErrorKind {
    ForbiddenAbi,
    ForbiddenAttr,
    ForbiddenItemKind(ItemKind),
    ForbiddenRepr(Vec<Repr>),
    ForbiddenType,
    MalformedAttrs,
    MissingPub,
    MissingRepr,
    MissingUnsafe,
    UnderscoreField,
    UnknownRepr,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForbiddenAbi => write!(f, "forbidden ABI"),
            Self::ForbiddenAttr => write!(f, "forbidden attribute"),
            Self::ForbiddenItemKind(ItemKind::Enum) => write!(
                f,
                "forbidden use of enum; use the `newtype_enum!` macro instead"
            ),
            Self::ForbiddenItemKind(_) => write!(f, "forbidden type of item"),
            Self::ForbiddenRepr(reprs) => {
                assert!(!reprs.is_empty());
                if reprs.len() == 1 {
                    write!(
                        f,
                        "the following repr attribute is forbidden: {:?}",
                        reprs[0]
                    )
                } else {
                    write!(
                        f,
                        "the following combination of repr attributes is forbidden: {reprs:?}"
                    )
                }
            }
            Self::ForbiddenType => write!(f, "forbidden type"),
            Self::MalformedAttrs => write!(f, "malformed attribute contents"),
            Self::MissingPub => write!(f, "missing pub"),
            Self::MissingRepr => write!(f, "missing repr"),
            Self::MissingUnsafe => write!(f, "missing unsafe"),
            Self::UnderscoreField => write!(f, "field name starts with `_`"),
            Self::UnknownRepr => write!(f, "unknown repr"),
        }
    }
}

/// Validation error type that includes the error location.
struct Error {
    kind: ErrorKind,
    path: PathBuf,
    line: usize,
    column: usize,
    code: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Format the error in the same way a compiler would, which allows
        // editors/IDEs to parse the source location.
        write!(
            f,
            "error: {}\n  --> {}:{}:{}\n{}",
            self.kind,
            self.path.display(),
            self.line,
            self.column + 1,
            self.code,
        )
    }
}

impl Error {
    fn new(kind: ErrorKind, path: &Path, spanned: &dyn Spanned) -> Self {
        let span = spanned.span();
        Self {
            kind,
            // Getting the source path from the span is not yet stable:
            // https://github.com/rust-lang/rust/issues/54725
            path: path.to_path_buf(),
            line: span.start().line,
            column: span.start().column,
            // This is `None` in unit tests.
            code: span.source_text().unwrap_or_default(),
        }
    }
}

/// True if the visibility is public without restriction (i.e. just `pub`, not
/// `pub(crate)` or similar).
fn is_pub(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

/// Type repr. A type may have more than one of these (e.g. both `C` and `packed`).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
enum Repr {
    Align(usize),
    C,
    Packed,
    Rust,
    Transparent,
}

/// A restricted view of `Attribute`, limited to just the attributes that are
/// expected in `uefi-raw`.
#[derive(Debug, Clone, Copy)]
enum ParsedAttr {
    Derive,
    Doc,
    Repr(Repr),
}

/// Parse `attrs` into a list of the more restricted `ParsedAttr` enum.
fn parse_attrs(attrs: &[Attribute], src: &Path) -> Result<Vec<ParsedAttr>, Error> {
    let mut va = Vec::new();
    for attr in attrs {
        let path = attr.path();

        if path.is_ident("derive") {
            va.push(ParsedAttr::Derive);
        } else if path.is_ident("doc") {
            va.push(ParsedAttr::Doc);
        } else if path.is_ident("repr") {
            let mut unknown_repr_found = false;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("C") {
                    va.push(ParsedAttr::Repr(Repr::C));
                } else if meta.path.is_ident("packed") {
                    va.push(ParsedAttr::Repr(Repr::Packed));
                } else if meta.path.is_ident("Rust") {
                    va.push(ParsedAttr::Repr(Repr::Rust));
                } else if meta.path.is_ident("transparent") {
                    va.push(ParsedAttr::Repr(Repr::Transparent));
                } else if meta.path.is_ident("align") {
                    let content;
                    parenthesized!(content in meta.input);
                    let lit: LitInt = content.parse()?;
                    let num = lit.base10_parse()?;
                    va.push(ParsedAttr::Repr(Repr::Align(num)));
                } else {
                    unknown_repr_found = true;
                }
                Ok(())
            })
            .map_err(|_| Error::new(ErrorKind::MalformedAttrs, src, attr))?;
            if unknown_repr_found {
                return Err(Error::new(ErrorKind::UnknownRepr, src, attr));
            }
        } else {
            return Err(Error::new(ErrorKind::ForbiddenAttr, src, attr));
        }
    }
    Ok(va)
}

/// Get a sorted list of all reprs from attributes.
fn get_reprs(attrs: &[ParsedAttr]) -> Vec<Repr> {
    let mut reprs: Vec<_> = attrs
        .iter()
        .filter_map(|attr| {
            if let ParsedAttr::Repr(repr) = attr {
                Some(*repr)
            } else {
                None
            }
        })
        .collect();
    reprs.sort();
    reprs
}

/// True if the function is `extern efiapi`.
fn is_efiapi(f: &TypeBareFn) -> bool {
    if let Some(Abi {
        name: Some(name), ..
    }) = &f.abi
    {
        if name.value() == "efiapi" {
            return true;
        }
    }
    false
}

/// Validate a type (used for fields, arguments, and return types).
fn check_type(ty: &Type, src: &Path) -> Result<(), Error> {
    match ty {
        Type::Array(TypeArray { elem, .. }) => check_type(elem, src),
        Type::BareFn(f) => check_fn_ptr(f, src),
        Type::Never(_) | Type::Path(_) => {
            // Allow.
            Ok(())
        }
        Type::Ptr(TypePtr { elem, .. }) => check_type(elem, src),
        ty => Err(Error::new(ErrorKind::ForbiddenType, src, ty)),
    }
}

/// Validate a function pointer.
fn check_fn_ptr(f: &TypeBareFn, src: &Path) -> Result<(), Error> {
    // Require `extern efiapi`, except for c-variadics.
    if !is_efiapi(f) && f.variadic.is_none() {
        return Err(Error::new(ErrorKind::ForbiddenAbi, src, f));
    }

    // Require `unsafe`.
    if f.unsafety.is_none() {
        return Err(Error::new(ErrorKind::MissingUnsafe, src, f));
    }

    // Validate argument types.
    for arg in &f.inputs {
        check_type(&arg.ty, src)?;
    }

    // Validate return type.
    match &f.output {
        ReturnType::Default => {}
        ReturnType::Type(_, output) => check_type(output, src)?,
    }

    Ok(())
}

/// Validate all struct fields. This is used for both named and unnamed fields.
fn check_fields(fields: &Punctuated<Field, Comma>, src: &Path) -> Result<(), Error> {
    for field in fields {
        // Ensure field is public.
        if !is_pub(&field.vis) {
            return Err(Error::new(ErrorKind::MissingPub, src, field));
        }

        // Ensure field name doesn't start with `_`.
        if let Some(ident) = &field.ident {
            if ident.to_string().starts_with('_') {
                return Err(Error::new(ErrorKind::UnderscoreField, src, ident));
            }
        }

        // Ensure a valid field type.
        check_type(&field.ty, src)?;
    }
    Ok(())
}

/// List with allowed combinations of representations (see [`Repr`]).
const ALLOWED_REPRS: &[&[Repr]] = &[&[Repr::C], &[Repr::C, Repr::Packed], &[Repr::Transparent]];

fn check_type_attrs(attrs: &[Attribute], spanned: &dyn Spanned, src: &Path) -> Result<(), Error> {
    let attrs = parse_attrs(attrs, src)?;
    let reprs = get_reprs(&attrs);

    if reprs.is_empty() {
        Err(Error::new(ErrorKind::MissingRepr, src, spanned))
    } else if ALLOWED_REPRS.contains(&reprs.as_slice()) {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::ForbiddenRepr(reprs), src, spanned))
    }
}

/// Validate a struct.
fn check_struct(item: &ItemStruct, src: &Path) -> Result<(), Error> {
    if !is_pub(&item.vis) {
        return Err(Error::new(ErrorKind::MissingPub, src, &item.struct_token));
    }

    match &item.fields {
        Fields::Named(FieldsNamed { named, .. }) => check_fields(named, src)?,
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => check_fields(unnamed, src)?,
        Fields::Unit => {}
    }

    check_type_attrs(&item.attrs, item, src)?;

    Ok(())
}

/// Validate a union.
fn check_union(item: &ItemUnion, src: &Path) -> Result<(), Error> {
    if !is_pub(&item.vis) {
        return Err(Error::new(ErrorKind::MissingPub, src, &item.union_token));
    }

    check_fields(&item.fields.named, src)?;

    check_type_attrs(&item.attrs, item, src)?;

    Ok(())
}

/// Validate a macro.
fn check_macro(item: &ItemMacro, src: &Path) -> Result<(), Error> {
    let mac = &item.mac;

    // Check that uses of the `bitflags` macro always set `repr(transparent)`.
    if mac.path.is_ident("bitflags") {
        // Parse just the attributes.
        struct Attrs(Vec<Attribute>);
        impl Parse for Attrs {
            fn parse(input: ParseStream) -> Result<Self, syn::Error> {
                let x = input.call(Attribute::parse_outer)?;
                let _: TokenStream = input.parse()?;
                Ok(Self(x))
            }
        }
        let attrs: Attrs = mac
            .parse_body()
            .map_err(|_| Error::new(ErrorKind::MalformedAttrs, src, mac))?;
        let attrs = parse_attrs(&attrs.0, src)?;

        let reprs = get_reprs(&attrs);
        let allowed_reprs: &[&[Repr]] = &[&[Repr::Transparent]];
        if !allowed_reprs.contains(&reprs.as_slice()) {
            return Err(Error::new(ErrorKind::ForbiddenRepr(reprs), src, mac));
        }
    }

    Ok(())
}

/// Validate a top-level item.
fn check_item(item: &Item, src: &Path) -> Result<(), Error> {
    match item {
        Item::Const(ItemConst { vis, ty, .. }) => {
            if !is_pub(vis) {
                return Err(Error::new(ErrorKind::MissingPub, src, item));
            }

            check_type(ty, src)?;
        }
        Item::Struct(item) => {
            check_struct(item, src)?;
        }
        Item::Union(item) => {
            check_union(item, src)?;
        }
        Item::Macro(item) => {
            check_macro(item, src)?;
        }
        Item::Type(ItemType { vis, .. }) => {
            if !is_pub(vis) {
                return Err(Error::new(ErrorKind::MissingPub, src, item));
            }
        }
        Item::Impl(_) | Item::Mod(_) | Item::Use(_) => {
            // Allow.
        }
        item => {
            return Err(Error::new(
                ErrorKind::ForbiddenItemKind(item.into()),
                src,
                item,
            ));
        }
    }

    Ok(())
}

/// Validate an entire source file.
fn check_file(src: &Path) -> Result<()> {
    println!("checking {}", src.display());

    let code = fs::read_to_string(src)?;
    let ast: File = syn::parse_str(&code)?;

    for item in ast.items.iter() {
        // Don't propagate check errors, instead format the error in the same
        // way as a compiler so that IDEs can parse it.
        if let Err(err) = check_item(item, src) {
            println!("{err}");
            process::exit(1);
        }
    }

    Ok(())
}

/// Validate the `uefi-raw` package.
pub fn check_raw() -> Result<()> {
    let package_path = Path::new("uefi-raw");
    assert!(package_path.exists());

    for entry in WalkDir::new(package_path) {
        let entry = entry?;
        let path = entry.path();

        if let Some(ext) = path.extension() {
            if ext == "rs" {
                check_file(path)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    fn src() -> &'static Path {
        Path::new("test")
    }

    #[track_caller]
    fn check_item_err(item: Item, expected_error: ErrorKind) {
        assert_eq!(check_item(&item, src()).unwrap_err().kind, expected_error);
    }

    #[test]
    fn test_invalid_item_enum() {
        // Rust enums are not allowed.
        check_item_err(
            parse_quote! {
                pub enum E {
                    A
                }
            },
            ErrorKind::ForbiddenItemKind(ItemKind::Enum),
        );
    }

    #[test]
    fn test_invalid_item_other() {
        // Top-level functions are not allowed.
        check_item_err(
            parse_quote! {
                pub fn x() {}
            },
            ErrorKind::ForbiddenItemKind(ItemKind::Other),
        );
    }

    #[test]
    fn test_macro() {
        // bitflags `repr` must be transparent.
        check_item_err(
            parse_quote! {
                bitflags! {
                    #[repr(C)]
                    pub struct Flags: u32 {
                        const A = 1;
                    }
                }
            },
            ErrorKind::ForbiddenRepr(vec![Repr::C]),
        );
    }

    #[test]
    fn test_fn_ptr() {
        let check_fn_err = |f, expected_error| {
            assert_eq!(check_fn_ptr(&f, src()).unwrap_err().kind, expected_error);
        };

        // Valid fn ptr.
        assert!(
            check_fn_ptr(
                &parse_quote! {
                    unsafe extern "efiapi" fn()
                },
                src(),
            )
            .is_ok()
        );

        // Valid fn ptr with c-variadics.
        assert!(
            check_fn_ptr(
                &parse_quote! {
                    unsafe extern "C" fn(usize, ...)
                },
                src(),
            )
            .is_ok()
        );

        // Not `extern efiapi`.
        check_fn_err(
            parse_quote! {
                unsafe extern "C" fn()
            },
            ErrorKind::ForbiddenAbi,
        );

        // Fn pointer is missing `unsafe`.
        check_fn_err(
            parse_quote! {
                extern "efiapi" fn()
            },
            ErrorKind::MissingUnsafe,
        );

        // Forbidden argument type: reference.
        check_fn_err(
            parse_quote! {
                unsafe extern "efiapi" fn(a: &u32)
            },
            ErrorKind::ForbiddenType,
        );

        // Forbidden return type: reference.
        check_fn_err(
            parse_quote! {
                unsafe extern "efiapi" fn() -> &u32
            },
            ErrorKind::ForbiddenType,
        );
    }

    #[test]
    fn test_struct() {
        // Valid struct.
        assert!(
            check_struct(
                &parse_quote! {
                    #[repr(C)]
                    pub struct S {
                        pub f: u32,
                    }
                },
                src(),
            )
            .is_ok()
        );

        // Missing `pub` on struct.
        check_item_err(
            parse_quote! {
                #[repr(C)]
                struct S {
                    pub f: u32,
                }
            },
            ErrorKind::MissingPub,
        );

        // Missing `pub` on field.
        check_item_err(
            parse_quote! {
                #[repr(C)]
                pub struct S {
                    f: u32,
                }
            },
            ErrorKind::MissingPub,
        );

        // Field name starts with `_`.
        check_item_err(
            parse_quote! {
                #[repr(C)]
                pub struct S {
                    pub _f: u32,
                }
            },
            ErrorKind::UnderscoreField,
        );

        // Missing `repr`.
        check_item_err(
            parse_quote! {
                pub struct S {
                    pub f: u32,
                }
            },
            ErrorKind::MissingRepr,
        );

        // Forbidden `repr`.
        check_item_err(
            parse_quote! {
                #[repr(Rust)]
                pub struct S {
                    pub f: u32,
                }
            },
            ErrorKind::ForbiddenRepr(vec![Repr::Rust]),
        );

        // Forbidden attr.
        check_item_err(
            parse_quote! {
                #[hello]
                #[repr(C)]
                pub struct S {
                    pub f: u32,
                }
            },
            ErrorKind::ForbiddenAttr,
        );

        // Forbidden field type: reference.
        check_item_err(
            parse_quote! {
                #[repr(C)]
                pub struct S {
                    pub f: &u32,
                }
            },
            ErrorKind::ForbiddenType,
        );
    }

    #[test]
    fn test_union() {
        // Valid union.
        assert!(
            check_union(
                &parse_quote! {
                    #[repr(C)]
                    pub union U {
                        pub a: u32,
                        pub b: u64,
                    }
                },
                src(),
            )
            .is_ok()
        );

        // Missing `pub` on union.
        check_item_err(
            parse_quote! {
                #[repr(C)]
                struct U {
                    pub f: u32,
                }
            },
            ErrorKind::MissingPub,
        );

        // Missing `pub` on field.
        check_item_err(
            parse_quote! {
                #[repr(C)]
                pub struct U {
                    f: u32,
                }
            },
            ErrorKind::MissingPub,
        );

        // Forbidden `repr`.
        check_item_err(
            parse_quote! {
                pub struct S {
                    pub f: u32,
                }
            },
            ErrorKind::MissingRepr,
        );
    }
}
