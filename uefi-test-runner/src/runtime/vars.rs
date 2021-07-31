use alloc::vec::Vec;
use log::info;
use uefi::prelude::*;
use uefi::table::runtime::VariableAttributes;
use uefi::{CStr16, Guid};

struct CString16(Vec<u16>);

impl CString16 {
    fn from_str(input: &str) -> CString16 {
        let mut v: Vec<u16> = input.encode_utf16().collect();
        v.push(0);
        CString16(v)
    }

    fn as_cstr16(&self) -> &CStr16 {
        match CStr16::from_u16_with_nul(&self.0) {
            Ok(s) => s,
            Err(_) => panic!("invalid string"),
        }
    }
}

fn test_variables(rt: &RuntimeServices) {
    let name = CString16::from_str("UefiRsTestVar");
    let test_value = b"TestValue";
    let test_attrs = VariableAttributes::BOOTSERVICE_ACCESS | VariableAttributes::RUNTIME_ACCESS;

    // Arbitrary GUID generated for this test.
    let vendor = Guid::from_values(
        0x9baf21cf,
        0xe187,
        0x497e,
        0xae77,
        [0x5b, 0xd8, 0xb0, 0xe0, 0x97, 0x03],
    );

    info!("Testing set_variable");
    rt.set_variable(name.as_cstr16(), &vendor, test_attrs, test_value)
        .expect_success("failed to set variable");

    info!("Testing get_variable_size");
    let size = rt
        .get_variable_size(name.as_cstr16(), &vendor)
        .expect_success("failed to get variable size");
    assert_eq!(size, test_value.len());

    info!("Testing get_variable");
    let mut buf = [0u8; 9];
    let (data, attrs) = rt
        .get_variable(name.as_cstr16(), &vendor, &mut buf)
        .expect_success("failed to get variable");
    assert_eq!(data, test_value);
    assert_eq!(attrs, test_attrs);
}

pub fn test(rt: &RuntimeServices) {
    test_variables(rt);
}
