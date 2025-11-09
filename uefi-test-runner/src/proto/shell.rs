// SPDX-License-Identifier: MIT OR Apache-2.0

use uefi::boot::{OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol};
use uefi::proto::device_path::DevicePath;
use uefi::proto::device_path::text::{AllowShortcuts, DisplayOnly};
use uefi::proto::shell::Shell;
use uefi::{Error, Status, boot, cstr16};
use uefi_raw::protocol::shell::ShellProtocol;

/// Test `current_dir()` and `set_current_dir()`
pub fn test_current_dir(shell: &ScopedProtocol<Shell>) {
    /* Test setting and getting current file system and current directory */
    let fs_var = cstr16!("fs0:");
    let dir_var = cstr16!("/");
    let status = shell.set_current_dir(Some(fs_var), Some(dir_var));
    assert!(status.is_ok());

    let cur_fs_str = shell
        .current_dir(Some(fs_var))
        .expect("Could not get the current file system mapping");
    let expected_fs_str = cstr16!("FS0:\\");
    assert_eq!(cur_fs_str, expected_fs_str);

    // Changing current file system
    let fs_var = cstr16!("fs1:");
    let dir_var = cstr16!("/");
    let status = shell.set_current_dir(Some(fs_var), Some(dir_var));
    assert!(status.is_ok());

    let cur_fs_str = shell
        .current_dir(Some(fs_var))
        .expect("Could not get the current file system mapping");
    assert_ne!(cur_fs_str, expected_fs_str);
    let expected_fs_str = cstr16!("FS1:\\");
    assert_eq!(cur_fs_str, expected_fs_str);

    // Changing current file system and current directory
    let fs_var = cstr16!("fs0:");
    let dir_var = cstr16!("efi/");
    let status = shell.set_current_dir(Some(fs_var), Some(dir_var));
    assert!(status.is_ok());

    let cur_fs_str = shell
        .current_dir(Some(fs_var))
        .expect("Could not get the current file system mapping");
    assert_ne!(cur_fs_str, expected_fs_str);
    let expected_fs_str = cstr16!("FS0:\\efi");
    assert_eq!(cur_fs_str, expected_fs_str);

    /* Test current working directory cases */

    // At this point, the current working file system has not been set
    // So we expect a NULL output
    assert!(shell.current_dir(None).is_err());
    assert_eq!(
        shell.current_dir(None).err().unwrap(),
        Error::new(Status::NOT_FOUND, ())
    );

    // Setting the current working file system and current working directory
    let dir_var = cstr16!("fs0:/");
    let status = shell.set_current_dir(None, Some(dir_var));
    assert!(status.is_ok());
    let cur_fs_str = shell
        .current_dir(Some(fs_var))
        .expect("Could not get the current file system mapping");
    let expected_fs_str = cstr16!("FS0:");
    assert_eq!(cur_fs_str, expected_fs_str);

    let cur_fs_str = shell
        .current_dir(None)
        .expect("Could not get the current file system mapping");
    assert_eq!(cur_fs_str, expected_fs_str);

    // Changing current working directory
    let dir_var = cstr16!("/efi");
    let status = shell.set_current_dir(None, Some(dir_var));
    assert!(status.is_ok());
    let cur_fs_str = shell
        .current_dir(Some(fs_var))
        .expect("Could not get the current file system mapping");
    let expected_fs_str = cstr16!("FS0:\\efi");
    assert_eq!(cur_fs_str, expected_fs_str);
    let cur_fs_str = shell
        .current_dir(None)
        .expect("Could not get the current file system mapping");
    assert_eq!(cur_fs_str, expected_fs_str);

    // Changing current directory in a non-current working file system
    let fs_var = cstr16!("fs0:");
    let dir_var = cstr16!("efi/tools");
    let status = shell.set_current_dir(Some(fs_var), Some(dir_var));
    assert!(status.is_ok());
    let cur_fs_str = shell
        .current_dir(None)
        .expect("Could not get the current file system mapping");
    assert_ne!(cur_fs_str, expected_fs_str);

    let expected_fs_str = cstr16!("FS0:\\efi\\tools");
    let cur_fs_str = shell
        .current_dir(Some(fs_var))
        .expect("Could not get the current file system mapping");
    assert_eq!(cur_fs_str, expected_fs_str);
}

/// Test `var()`, `vars()`, and `set_var()`
pub fn test_var(shell: &ScopedProtocol<Shell>) {
    /* Test retrieving list of environment variable names */
    let mut cur_env_vec = shell.vars();
    assert_eq!(cur_env_vec.next().unwrap().0, cstr16!("path"));
    // check pre-defined shell variables; see UEFI Shell spec
    assert_eq!(cur_env_vec.next().unwrap().0, cstr16!("nonesting"));
    let cur_env_vec = shell.vars();
    let default_len = cur_env_vec.count();

    /* Test setting and getting a specific environment variable */
    let test_var = cstr16!("test_var");
    let test_val = cstr16!("test_val");

    let found_var = shell.vars().any(|(env_var, _)| env_var == test_var);
    assert!(!found_var);
    assert!(shell.var(test_var).is_none());

    let status = shell.set_var(test_var, test_val, false);
    assert!(status.is_ok());
    let cur_env_str = shell
        .var(test_var)
        .expect("Could not get environment variable");
    assert_eq!(cur_env_str, test_val);

    let found_var = shell.vars().any(|(env_var, _)| env_var == test_var);
    assert!(found_var);
    let cur_env_vec = shell.vars();
    assert_eq!(cur_env_vec.count(), default_len + 1);

    /* Test deleting environment variable */
    let test_val = cstr16!("");
    let status = shell.set_var(test_var, test_val, false);
    assert!(status.is_ok());
    assert!(shell.var(test_var).is_none());

    let found_var = shell.vars().any(|(env_var, _)| env_var == test_var);
    assert!(!found_var);
    let cur_env_vec = shell.vars();
    assert_eq!(cur_env_vec.count(), default_len);
}

pub fn test() {
    info!("Running shell protocol tests");

    let handle = boot::get_handle_for_protocol::<Shell>().expect("No Shell handles");

    let shell =
        boot::open_protocol_exclusive::<Shell>(handle).expect("Failed to open Shell protocol");

    test_current_dir(&shell);
    test_var(&shell);
}
