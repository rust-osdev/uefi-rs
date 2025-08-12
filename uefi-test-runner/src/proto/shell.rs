// SPDX-License-Identifier: MIT OR Apache-2.0

use uefi::boot::ScopedProtocol;
use uefi::proto::shell::Shell;
use uefi::{CStr16, boot};
use uefi_raw::Status;

/// Test ``get_env()``, ``get_envs()``, and ``set_env()``
pub fn test_env(shell: &ScopedProtocol<Shell>) {
    let mut test_buf = [0u16; 128];

    /* Test retrieving list of environment variable names */
    let cur_env_vec = shell.get_envs();
    assert_eq!(
        *cur_env_vec.first().unwrap(),
        CStr16::from_str_with_buf("path", &mut test_buf).unwrap()
    );
    assert_eq!(
        *cur_env_vec.get(1).unwrap(),
        CStr16::from_str_with_buf("nonesting", &mut test_buf).unwrap()
    );
    let default_len = cur_env_vec.len();

    /* Test setting and getting a specific environment variable */
    let mut test_env_buf = [0u16; 32];
    let test_var = CStr16::from_str_with_buf("test_var", &mut test_env_buf).unwrap();
    let mut test_val_buf = [0u16; 32];
    let test_val = CStr16::from_str_with_buf("test_val", &mut test_val_buf).unwrap();
    assert!(shell.get_env(test_var).is_none());
    let status = shell.set_env(test_var, test_val, false);
    assert_eq!(status, Status::SUCCESS);
    let cur_env_str = shell
        .get_env(test_var)
        .expect("Could not get environment variable");
    assert_eq!(cur_env_str, test_val);

    assert!(!cur_env_vec.contains(&test_var));
    let cur_env_vec = shell.get_envs();
    assert!(cur_env_vec.contains(&test_var));
    assert_eq!(cur_env_vec.len(), default_len + 1);

    /* Test deleting environment variable */
    let test_val = CStr16::from_str_with_buf("", &mut test_val_buf).unwrap();
    let status = shell.set_env(test_var, test_val, false);
    assert_eq!(status, Status::SUCCESS);
    assert!(shell.get_env(test_var).is_none());

    let cur_env_vec = shell.get_envs();
    assert!(!cur_env_vec.contains(&test_var));
    assert_eq!(cur_env_vec.len(), default_len);
}

pub fn test() {
    info!("Running shell protocol tests");

    let handle = boot::get_handle_for_protocol::<Shell>().expect("No Shell handles");

    let shell =
        boot::open_protocol_exclusive::<Shell>(handle).expect("Failed to open Shell protocol");

    test_env(&shell);
}
