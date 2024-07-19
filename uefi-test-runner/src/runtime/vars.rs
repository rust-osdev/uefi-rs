use log::info;
use uefi::prelude::*;
use uefi::table::runtime::{VariableAttributes, VariableVendor};
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

fn test_variables(rt: &RuntimeServices) {
    info!("Testing set_variable");
    rt.set_variable(NAME, VENDOR, ATTRS, VALUE)
        .expect("failed to set variable");

    info!("Testing get_variable_size");
    let size = rt
        .get_variable_size(NAME, VENDOR)
        .expect("failed to get variable size");
    assert_eq!(size, VALUE.len());

    info!("Testing get_variable");
    let mut buf = [0u8; 9];
    let (data, attrs) = rt
        .get_variable(NAME, VENDOR, &mut buf)
        .expect("failed to get variable");
    assert_eq!(data, VALUE);
    assert_eq!(attrs, ATTRS);

    info!("Testing get_variable_boxed");
    let (data, attrs) = rt
        .get_variable_boxed(NAME, VENDOR)
        .expect("failed to get variable");
    assert_eq!(&*data, VALUE);
    assert_eq!(attrs, ATTRS);

    info!("Testing variable_keys");
    let variable_keys = rt.variable_keys().expect("failed to get variable keys");
    info!("Found {} variables", variable_keys.len());
    // There are likely a bunch of variables, only print out the first one
    // during the test to avoid spamming the log.
    if let Some(key) = variable_keys.first() {
        info!("First variable: {}", key);
    }

    info!("Testing delete_variable()");
    rt.delete_variable(NAME, VENDOR)
        .expect("failed to delete variable");
    assert_eq!(
        rt.get_variable(NAME, VENDOR, &mut buf)
            .unwrap_err()
            .status(),
        Status::NOT_FOUND
    );
}

/// Test the variable functions in `uefi::runtime`.
fn test_variables_freestanding() {
    // Create the test variable.
    runtime::set_variable(NAME, VENDOR, ATTRS, VALUE).expect("failed to set variable");

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

    // Delete the variable and verify it can no longer be read.
    runtime::delete_variable(NAME, VENDOR).expect("failed to delete variable");
    assert_eq!(
        runtime::get_variable(NAME, VENDOR, &mut buf)
            .unwrap_err()
            .status(),
        Status::NOT_FOUND
    );
}

fn test_variable_info(rt: &RuntimeServices) {
    info!(
        "Storage for non-volatile boot-services variables: {:?}",
        rt.query_variable_info(
            VariableAttributes::BOOTSERVICE_ACCESS | VariableAttributes::NON_VOLATILE
        )
        .unwrap(),
    );
    info!(
        "Storage for volatile runtime variables: {:?}",
        rt.query_variable_info(
            VariableAttributes::BOOTSERVICE_ACCESS | VariableAttributes::RUNTIME_ACCESS
        )
        .unwrap(),
    );
}

pub fn test(rt: &RuntimeServices) {
    test_variables(rt);
    test_variable_info(rt);
    test_variables_freestanding();
}
