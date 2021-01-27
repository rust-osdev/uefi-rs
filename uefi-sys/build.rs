use bindgen::*;
use std::{env::*, path::*};

#[cfg(target_arch = "x86_64")]
fn platform_specific_include() -> &'static str {
    "X64"
}

#[cfg(target_arch = "aarch64")]
fn platform_specific_include() -> &'static str {
    "AArch64"
}

#[cfg(target_arch = "i686")]
fn platform_specific_include() -> &'static str {
    "Ia32"
}

fn generate_bindings() {
    let out_path = PathBuf::from(
        var_os("OUT_DIR")
            .expect("OUT_DIR environment variable is required to generate the bindings."),
    );

    let base_include_dir = PathBuf::from("./external/edk2/MdePkg/Include");

    let bindings = Builder::default()
        .header("src/uefi_spec.h")
        .use_core()
        .layout_tests(false)
        .ctypes_prefix("cty")
        .rust_target(RustTarget::Nightly)
        .derive_debug(false)
        .impl_debug(true)
        // bindgen issue?
        .blacklist_type("EFI_BOOT_KEY_DATA__bindgen_ty_1")
        // variadic not supported with efiapi abi?
        .opaque_type("EFI_INSTALL_MULTIPLE_PROTOCOL_INTERFACES")
        .opaque_type("EFI_UNINSTALL_MULTIPLE_PROTOCOL_INTERFACES")
        .clang_arg(format!(
            "-I{}",
            base_include_dir
                .to_str()
                .expect("UTF-8 error on include path")
        ))
        .clang_arg(format!(
            "-I{}",
            base_include_dir
                .join(platform_specific_include())
                .to_str()
                .expect("UTF-8 error on include path")
        ))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Failed to generate bindings.");

    fn add_derive_clause(struct_name: &str, input: String) -> String {
        let src = format!("pub struct {}", struct_name);
        let dst = format!(
            "#[derive(Debug, PartialEq, Eq)]\npub struct {}",
            struct_name
        );
        input.replace(&src, &dst)
    }

    let derive_debug_eq_types = [
        "GUID",
        "EFI_TABLE_HEADER",
        "EFI_CONFIGURATION_TABLE",
        "EFI_MEMORY_DESCRIPTOR",
        "EFI_TIME_CAPABILITIES",
        "EFI_SIMPLE_TEXT_OUTPUT_MODE",
        "EFI_SIMPLE_POINTER_STATE",
        "EFI_SIMPLE_POINTER_MODE",
        "EFI_PIXEL_BITMASK",
        "EFI_GRAPHICS_OUTPUT_BLT_PIXEL",
        "EFI_GRAPHICS_OUTPUT_MODE_INFORMATION",
        "EFI_SERIAL_IO_MODE",
    ];

    // common bindgen, give us a callback in which we can proceed to types manipulation...
    let raw_file_content = derive_debug_eq_types
        .iter()
        .fold(bindings.to_string(), |output, struct_name| {
            add_derive_clause(struct_name, output)
        });

    std::fs::write(out_path.join("uefi_spec.rs"), raw_file_content)
        .expect("Could not write uefi_spec.rs");
}

fn main() {
    println!("cargo:rerun-if-changed=src/uefi_spec.h");
    generate_bindings();
}
