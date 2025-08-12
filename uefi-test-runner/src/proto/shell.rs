// SPDX-License-Identifier: MIT OR Apache-2.0

use uefi::boot::ScopedProtocol;
use uefi::proto::shell::Shell;
use uefi::{boot, cstr16};
use uefi_raw::Status;

/// Test `get_env()`, `get_envs()`, and `set_env()`
pub fn test_env(shell: &ScopedProtocol<Shell>) {
    /* Test retrieving list of environment variable names */
    let mut cur_env_vec = shell.get_envs();
    assert_eq!(cur_env_vec.next().unwrap(), cstr16!("path"),);
    // check pre-defined shell variables; see UEFI Shell spec
    assert_eq!(cur_env_vec.next().unwrap(), cstr16!("nonesting"),);
    let cur_env_vec = shell.get_envs();
    let default_len = cur_env_vec.count();

    /* Test setting and getting a specific environment variable */
    let cur_env_vec = shell.get_envs();
    let test_var = cstr16!("test_var");
    let test_val = cstr16!("test_val");
    assert!(shell.get_env(test_var).is_none());
    let status = shell.set_env(test_var, test_val, false);
    assert_eq!(status, Status::SUCCESS);
    let cur_env_str = shell
        .get_env(test_var)
        .expect("Could not get environment variable");
    assert_eq!(cur_env_str, test_val);

    let mut found_var = false;
    for env_var in cur_env_vec {
        if env_var == test_var {
            found_var = true;
        }
    }
    assert!(!found_var);
    let cur_env_vec = shell.get_envs();
    let mut found_var = false;
    for env_var in cur_env_vec {
        if env_var == test_var {
            found_var = true;
        }
    }
    assert!(found_var);

    let cur_env_vec = shell.get_envs();
    assert_eq!(cur_env_vec.count(), default_len + 1);

    /* Test deleting environment variable */
    let test_val = cstr16!("");
    let status = shell.set_env(test_var, test_val, false);
    assert_eq!(status, Status::SUCCESS);
    assert!(shell.get_env(test_var).is_none());

    let cur_env_vec = shell.get_envs();
    let mut found_var = false;
    for env_var in cur_env_vec {
        if env_var == test_var {
            found_var = true;
        }
    }
    assert!(!found_var);
    let cur_env_vec = shell.get_envs();
    assert_eq!(cur_env_vec.count(), default_len);
}

pub fn test() {
    info!("Running shell protocol tests");

    let handle = boot::get_handle_for_protocol::<Shell>().expect("No Shell handles");

    let shell =
        boot::open_protocol_exclusive::<Shell>(handle).expect("Failed to open Shell protocol");

    test_env(&shell);
}
