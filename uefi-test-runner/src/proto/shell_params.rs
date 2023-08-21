use uefi::proto::shell_params::ShellParameters;
use uefi::table::boot::BootServices;
use uefi::CStr16;

pub fn test(bt: &BootServices) {
    info!("Running loaded image protocol test");

    let image = bt
        .get_handle_for_protocol::<ShellParameters>()
        .expect("No ShellParameters handles");
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

    // Was run as: shell.efi test_runner.efi
    assert_eq!(shell_params.argc, 2);
}
