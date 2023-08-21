use alloc::string::String;
use alloc::vec::Vec;
use log::info;
use uefi::prelude::*;
use uefi::proto::device_path::build::{self, DevicePathBuilder};
use uefi::proto::device_path::{DevicePath, DeviceSubType, DeviceType, LoadedImageDevicePath};
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::shell_params::ShellParameters;
use uefi::table::boot::{BootServices, LoadImageSource};
use uefi::CStr16;

pub fn test(image: Handle, bt: &BootServices) {
    info!("Running loaded image protocol test");

    let shell_params = bt
        .open_protocol_exclusive::<ShellParameters>(image)
        .expect("Failed to open ShellParameters protocol");

    info!("Argc: {}", shell_params.argc);
    info!("Args:");
    for arg in shell_params.get_args_slice() {
        let arg_str = unsafe { CStr16::from_ptr(*arg) };
        info!("  '{}'", arg_str);
    }

    assert_eq!(shell_params.argc, shell_params.get_args_slice().len());

    // By default a single argument, the executable's path
    assert_eq!(shell_params.argc, 1);

    subshell_runner(image, bt);
}

/// Get the device path of the shell app. This is the same as the
/// currently-loaded image's device path, but with the file path part changed.
fn get_shell_app_device_path<'a>(
    boot_services: &BootServices,
    storage: &'a mut Vec<u8>,
) -> &'a DevicePath {
    let loaded_image_device_path = boot_services
        .open_protocol_exclusive::<LoadedImageDevicePath>(boot_services.image_handle())
        .expect("failed to open LoadedImageDevicePath protocol");

    let mut builder = DevicePathBuilder::with_vec(storage);
    for node in loaded_image_device_path.node_iter() {
        if node.full_type() == (DeviceType::MEDIA, DeviceSubType::MEDIA_FILE_PATH) {
            break;
        }
        builder = builder.push(&node).unwrap();
    }
    builder = builder
        .push(&build::media::FilePath {
            path_name: cstr16!(r"efi\boot\shell.efi"),
        })
        .unwrap();
    builder.finalize().unwrap()
}

fn subshell_runner(image: Handle, boot_services: &BootServices) {
    let mut storage = Vec::new();
    let shell_image_path = get_shell_app_device_path(boot_services, &mut storage);

    // Load the shell app.
    let shell_image_handle = boot_services
        .load_image(
            image,
            LoadImageSource::FromDevicePath {
                device_path: shell_image_path,
                from_boot_manager: false,
            },
        )
        .expect("failed to load shell app");

    // Set the command line passed to the shell app so that it will run the
    // test-runner app. This automatically turns off the five-second delay.
    let mut shell_loaded_image = boot_services
        .open_protocol_exclusive::<LoadedImage>(shell_image_handle)
        .expect("failed to open LoadedImage protocol");
    let load_options = cstr16!(r"shell.efi test_runner.efi arg1 arg2");
    unsafe {
        shell_loaded_image.set_load_options(
            load_options.as_ptr().cast(),
            load_options.num_bytes() as u32,
        );
    }

    info!("launching the sub shell app");
    boot_services
        .start_image(shell_image_handle)
        .expect("failed to launch the shell app");
}

pub fn test_subshell(image: Handle, boot_services: &BootServices) {
    info!("Running test from subshell");

    let shell_params = boot_services
        .open_protocol_exclusive::<ShellParameters>(image)
        .expect("Failed to open ShellParameters protocol");

    info!("Argc: {}", shell_params.argc);
    info!("Args:");
    for arg in shell_params.get_args_slice() {
        let arg_str = unsafe { CStr16::from_ptr(*arg) };
        info!("  '{}'", arg_str);
    }

    assert_eq!(shell_params.argc, shell_params.get_args_slice().len());

    let args: Vec<String> = shell_params.get_args().collect();
    assert_eq!(args, vec![r"FS0:\efi\boot\test_runner.efi", "arg1", "arg2"]);

    // test_runner.efi arg1 arg2
    assert_eq!(shell_params.argc, 3);
}
