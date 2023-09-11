use uefi::proto::shell_params::ShellParameters;
use uefi::table::boot::BootServices;

use alloc::string::ToString;
use alloc::vec::Vec;

pub fn test(bt: &BootServices) {
    info!("Running loaded image protocol test");

    let image = bt
        .get_handle_for_protocol::<ShellParameters>()
        .expect("No ShellParameters handles");
    let shell_params = bt
        .open_protocol_exclusive::<ShellParameters>(image)
        .expect("Failed to open ShellParameters protocol");

    assert_eq!(shell_params.args_len(), 4);
    assert_eq!(
        shell_params
            .args()
            .map(|x| x.to_string())
            .collect::<Vec<_>>(),
        &["shell.efi", "test_runner.efi", "arg1", "arg2"]
    );
}
