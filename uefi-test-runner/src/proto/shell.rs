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

/// Test ``get_cur_dir()`` and ``set_cur_dir()``
pub fn test_cur_dir(shell: &ScopedProtocol<Shell>) {
    let mut test_buf = [0u16; 128];

    /* Test setting and getting current file system and current directory */
    let mut fs_buf = [0u16; 16];
    let fs_var = CStr16::from_str_with_buf("fs0:", &mut fs_buf).unwrap();
    let mut dir_buf = [0u16; 32];
    let dir_var = CStr16::from_str_with_buf("/", &mut dir_buf).unwrap();
    let status = shell.set_cur_dir(Some(fs_var), Some(dir_var));
    assert_eq!(status, Status::SUCCESS);

    let cur_fs_str = shell
        .get_cur_dir(Some(fs_var))
        .expect("Could not get the current file system mapping");
    let expected_fs_str = CStr16::from_str_with_buf("FS0:\\", &mut test_buf).unwrap();
    assert_eq!(cur_fs_str, expected_fs_str);

    // Changing current file system
    let fs_var = CStr16::from_str_with_buf("fs1:", &mut fs_buf).unwrap();
    let dir_var = CStr16::from_str_with_buf("/", &mut dir_buf).unwrap();
    let status = shell.set_cur_dir(Some(fs_var), Some(dir_var));
    assert_eq!(status, Status::SUCCESS);

    let cur_fs_str = shell
        .get_cur_dir(Some(fs_var))
        .expect("Could not get the current file system mapping");
    assert_ne!(cur_fs_str, expected_fs_str);
    let expected_fs_str = CStr16::from_str_with_buf("FS1:\\", &mut test_buf).unwrap();
    assert_eq!(cur_fs_str, expected_fs_str);

    // Changing current file system and current directory
    let fs_var = CStr16::from_str_with_buf("fs0:", &mut fs_buf).unwrap();
    let dir_var = CStr16::from_str_with_buf("efi/", &mut dir_buf).unwrap();
    let status = shell.set_cur_dir(Some(fs_var), Some(dir_var));
    assert_eq!(status, Status::SUCCESS);

    let cur_fs_str = shell
        .get_cur_dir(Some(fs_var))
        .expect("Could not get the current file system mapping");
    assert_ne!(cur_fs_str, expected_fs_str);
    let expected_fs_str = CStr16::from_str_with_buf("FS0:\\efi", &mut test_buf).unwrap();
    assert_eq!(cur_fs_str, expected_fs_str);

    /* Test current working directory cases */

    // At this point, the current working file system has not been set
    // So we expect a NULL output
    assert!(shell.get_cur_dir(None).is_none());

    // Setting the current working file system and current working directory
    let dir_var = CStr16::from_str_with_buf("fs0:/", &mut dir_buf).unwrap();
    let status = shell.set_cur_dir(None, Some(dir_var));
    assert_eq!(status, Status::SUCCESS);
    let cur_fs_str = shell
        .get_cur_dir(Some(fs_var))
        .expect("Could not get the current file system mapping");
    let expected_fs_str = CStr16::from_str_with_buf("FS0:", &mut test_buf).unwrap();
    assert_eq!(cur_fs_str, expected_fs_str);

    let cur_fs_str = shell
        .get_cur_dir(None)
        .expect("Could not get the current file system mapping");
    assert_eq!(cur_fs_str, expected_fs_str);

    // Changing current working directory
    let dir_var = CStr16::from_str_with_buf("/efi", &mut dir_buf).unwrap();
    let status = shell.set_cur_dir(None, Some(dir_var));
    assert_eq!(status, Status::SUCCESS);
    let cur_fs_str = shell
        .get_cur_dir(Some(fs_var))
        .expect("Could not get the current file system mapping");
    let expected_fs_str = CStr16::from_str_with_buf("FS0:\\efi", &mut test_buf).unwrap();
    assert_eq!(cur_fs_str, expected_fs_str);
    let cur_fs_str = shell
        .get_cur_dir(None)
        .expect("Could not get the current file system mapping");
    assert_eq!(cur_fs_str, expected_fs_str);

    // Changing current directory in a non-current working file system
    let fs_var = CStr16::from_str_with_buf("fs0:", &mut fs_buf).unwrap();
    let dir_var = CStr16::from_str_with_buf("efi/tools", &mut dir_buf).unwrap();
    let status = shell.set_cur_dir(Some(fs_var), Some(dir_var));
    assert_eq!(status, Status::SUCCESS);
    let cur_fs_str = shell
        .get_cur_dir(None)
        .expect("Could not get the current file system mapping");
    assert_ne!(cur_fs_str, expected_fs_str);

    let expected_fs_str = CStr16::from_str_with_buf("FS0:\\efi\\tools", &mut test_buf).unwrap();
    let cur_fs_str = shell
        .get_cur_dir(Some(fs_var))
        .expect("Could not get the current file system mapping");
    assert_eq!(cur_fs_str, expected_fs_str);
}

pub fn test() {
    info!("Running shell protocol tests");

    let handle = boot::get_handle_for_protocol::<Shell>().expect("No Shell handles");

    let shell =
        boot::open_protocol_exclusive::<Shell>(handle).expect("Failed to open Shell protocol");

    test_env(&shell);
    test_cur_dir(&shell);
}
