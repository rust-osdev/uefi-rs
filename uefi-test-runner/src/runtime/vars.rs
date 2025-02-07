// SPDX-License-Identifier: MIT OR Apache-2.0

use log::info;
use uefi::prelude::*;
use uefi::runtime::{VariableAttributes, VariableVendor};
use uefi::{guid, runtime, CStr16, Error};

/// Test variable name.
const NAME: &CStr16 = cstr16!("UefiRsTestVar");

/// Test variable vendor.
const VENDOR: &VariableVendor = &VariableVendor(guid!("9baf21cf-e187-497e-ae77-5bd8b0e09703"));

/// Test variable value.
const VALUE: &[u8] = b"TestValue";

/// Test variable attributes.
const ATTRS: VariableAttributes =
    VariableAttributes::BOOTSERVICE_ACCESS.union(VariableAttributes::RUNTIME_ACCESS);

/// Test the variable functions in `uefi::runtime`.
fn test_variables() {
    assert!(!runtime::variable_exists(NAME, VENDOR).unwrap());

    // Create the test variable.
    runtime::set_variable(NAME, VENDOR, ATTRS, VALUE).expect("failed to set variable");

    assert!(runtime::variable_exists(NAME, VENDOR).unwrap());

    // Test `get_variable` with too small of a buffer.
    let mut buf = [0u8; 0];
    assert_eq!(
        runtime::get_variable(NAME, VENDOR, &mut buf).unwrap_err(),
        Error::new(Status::BUFFER_TOO_SMALL, Some(9))
    );

    // Test `get_variable`.
    let mut buf = [0u8; 9];
    let (data, attrs) =
        runtime::get_variable(NAME, VENDOR, &mut buf).expect("failed to get variable");
    assert_eq!(data, VALUE);
    assert_eq!(attrs, ATTRS);

    // Test `get_variable_boxed`.
    let (data, attrs) = runtime::get_variable_boxed(NAME, VENDOR).expect("failed to get variable");
    assert_eq!(&*data, VALUE);
    assert_eq!(attrs, ATTRS);

    // Test that the variable is present in the `variable_keys` iterator.
    let find_by_key = || {
        runtime::variable_keys().any(|k| {
            let k = k.as_ref().unwrap();
            k.name == NAME && &k.vendor == VENDOR
        })
    };
    assert!(find_by_key());

    // Delete the variable and verify it can no longer be read.
    runtime::delete_variable(NAME, VENDOR).expect("failed to delete variable");
    assert!(!runtime::variable_exists(NAME, VENDOR).unwrap());
    assert_eq!(
        runtime::get_variable(NAME, VENDOR, &mut buf)
            .unwrap_err()
            .status(),
        Status::NOT_FOUND
    );
    // Variable is no longer present in the `variable_keys` iterator.
    assert!(!find_by_key());
}

fn test_variable_info() {
    let attr = VariableAttributes::BOOTSERVICE_ACCESS | VariableAttributes::NON_VOLATILE;
    let info = runtime::query_variable_info(attr).unwrap();
    info!("Storage for non-volatile boot-services variables: {info:?}");

    let attr = VariableAttributes::BOOTSERVICE_ACCESS | VariableAttributes::RUNTIME_ACCESS;
    let info = runtime::query_variable_info(attr).unwrap();
    info!("Storage for volatile runtime variables: {info:?}");
}

pub fn test() {
    test_variable_info();
    test_variables();
}
