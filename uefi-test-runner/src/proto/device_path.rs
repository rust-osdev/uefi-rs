// SPDX-License-Identifier: MIT OR Apache-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;
use uefi::proto::device_path::build::{self, DevicePathBuilder};
use uefi::proto::device_path::text::{
    AllowShortcuts, DevicePathFromText, DevicePathToText, DisplayOnly,
};
use uefi::proto::device_path::{messaging, DevicePath, DevicePathNode, LoadedImageDevicePath};
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::media::disk::DiskIo;
use uefi::{boot, cstr16};

pub fn test() {
    info!("Running device path tests");

    test_convert_device_path_to_text();
    test_device_path_to_string();

    test_convert_device_node_to_text();
    test_device_path_node_to_string();

    test_convert_text_to_device_path();
    test_convert_text_to_device_node();

    test_device_path_append();

    // Get the current executable's device path via the `LoadedImage` protocol.
    let loaded_image = boot::open_protocol_exclusive::<LoadedImage>(boot::image_handle()).unwrap();
    let device_path =
        boot::open_protocol_exclusive::<DevicePath>(loaded_image.device().unwrap()).unwrap();

    // Get the `LoadedImageDevicePath`. Verify it start with the same nodes as
    // `device_path`.
    let loaded_image_device_path =
        boot::open_protocol_exclusive::<LoadedImageDevicePath>(boot::image_handle()).unwrap();
    for (n1, n2) in device_path
        .node_iter()
        .zip(loaded_image_device_path.node_iter())
    {
        assert_eq!(n1, n2);
    }

    // Test finding a handle by device path.
    let mut dp = &*device_path;
    boot::locate_device_path::<DiskIo>(&mut dp).unwrap();
}

fn create_test_device_path() -> Box<DevicePath> {
    let mut v = Vec::new();
    DevicePathBuilder::with_vec(&mut v)
        // Add an ATAPI node because edk2 displays it differently depending on
        // the value of `DisplayOnly`.
        .push(&build::messaging::Atapi {
            primary_secondary: messaging::PrimarySecondary::PRIMARY,
            master_slave: messaging::MasterSlave::MASTER,
            logical_unit_number: 1,
        })
        .unwrap()
        // Add a messaging::vendor node because edk2 displays it differently
        // depending on the value of `AllowShortcuts`.
        .push(&build::messaging::Vendor {
            vendor_guid: messaging::Vendor::PC_ANSI,
            vendor_defined_data: &[],
        })
        .unwrap()
        .finalize()
        .unwrap()
        .to_boxed()
}

/// Test `DevicePathToText::convert_device_path_to_text`.
fn test_convert_device_path_to_text() {
    let path = create_test_device_path();

    let proto = boot::open_protocol_exclusive::<DevicePathToText>(
        boot::get_handle_for_protocol::<DevicePathToText>().unwrap(),
    )
    .unwrap();

    let to_text = |display_only, allow_shortcuts| {
        proto
            .convert_device_path_to_text(&path, display_only, allow_shortcuts)
            .unwrap()
    };

    assert_eq!(
        &*to_text(DisplayOnly(true), AllowShortcuts(true)),
        cstr16!("Ata(0x1)/VenPcAnsi()")
    );
    assert_eq!(
        &*to_text(DisplayOnly(true), AllowShortcuts(false)),
        cstr16!("Ata(0x1)/VenMsg(E0C14753-F9BE-11D2-9A0C-0090273FC14D)")
    );
    assert_eq!(
        &*to_text(DisplayOnly(false), AllowShortcuts(true)),
        cstr16!("Ata(Primary,Master,0x1)/VenPcAnsi()")
    );
    assert_eq!(
        &*to_text(DisplayOnly(false), AllowShortcuts(false)),
        cstr16!("Ata(Primary,Master,0x1)/VenMsg(E0C14753-F9BE-11D2-9A0C-0090273FC14D)")
    );
}

/// Test `DevicePath::to_string`.
fn test_device_path_to_string() {
    let path = create_test_device_path();

    let to_text =
        |display_only, allow_shortcuts| path.to_string(display_only, allow_shortcuts).unwrap();

    assert_eq!(
        &*to_text(DisplayOnly(true), AllowShortcuts(true)),
        cstr16!("Ata(0x1)/VenPcAnsi()")
    );
    assert_eq!(
        &*to_text(DisplayOnly(true), AllowShortcuts(false)),
        cstr16!("Ata(0x1)/VenMsg(E0C14753-F9BE-11D2-9A0C-0090273FC14D)")
    );
    assert_eq!(
        &*to_text(DisplayOnly(false), AllowShortcuts(true)),
        cstr16!("Ata(Primary,Master,0x1)/VenPcAnsi()")
    );
    assert_eq!(
        &*to_text(DisplayOnly(false), AllowShortcuts(false)),
        cstr16!("Ata(Primary,Master,0x1)/VenMsg(E0C14753-F9BE-11D2-9A0C-0090273FC14D)")
    );
}

/// Test `DevicePathToText::convert_device_node_to_text`.
fn test_convert_device_node_to_text() {
    let path = create_test_device_path();
    let nodes: Vec<_> = path.node_iter().collect();

    let proto = boot::open_protocol_exclusive::<DevicePathToText>(
        boot::get_handle_for_protocol::<DevicePathToText>().unwrap(),
    )
    .unwrap();

    let to_text = |node, display_only, allow_shortcuts| {
        proto
            .convert_device_node_to_text(node, display_only, allow_shortcuts)
            .unwrap()
    };

    assert_eq!(
        &*to_text(nodes[0], DisplayOnly(true), AllowShortcuts(true)),
        cstr16!("Ata(0x1)")
    );
    assert_eq!(
        &*to_text(nodes[0], DisplayOnly(false), AllowShortcuts(true)),
        cstr16!("Ata(Primary,Master,0x1)")
    );
    assert_eq!(
        &*to_text(nodes[1], DisplayOnly(false), AllowShortcuts(true)),
        cstr16!("VenPcAnsi()")
    );
    assert_eq!(
        &*to_text(nodes[1], DisplayOnly(false), AllowShortcuts(false)),
        cstr16!("VenMsg(E0C14753-F9BE-11D2-9A0C-0090273FC14D)")
    );
}

/// Test `DevicePathNode::to_string`.
fn test_device_path_node_to_string() {
    let path = create_test_device_path();
    let nodes: Vec<_> = path.node_iter().collect();

    let to_text = |node: &DevicePathNode, display_only, allow_shortcuts| {
        node.to_string(display_only, allow_shortcuts).unwrap()
    };

    assert_eq!(
        &*to_text(nodes[0], DisplayOnly(true), AllowShortcuts(true)),
        cstr16!("Ata(0x1)")
    );
    assert_eq!(
        &*to_text(nodes[0], DisplayOnly(false), AllowShortcuts(true)),
        cstr16!("Ata(Primary,Master,0x1)")
    );
    assert_eq!(
        &*to_text(nodes[1], DisplayOnly(false), AllowShortcuts(true)),
        cstr16!("VenPcAnsi()")
    );
    assert_eq!(
        &*to_text(nodes[1], DisplayOnly(false), AllowShortcuts(false)),
        cstr16!("VenMsg(E0C14753-F9BE-11D2-9A0C-0090273FC14D)")
    );
}

/// Test `DevicePathFromText::convert_text_to_device_path`.
fn test_convert_text_to_device_path() {
    let text = cstr16!("Ata(Primary,Master,0x1)/VenMsg(E0C14753-F9BE-11D2-9A0C-0090273FC14D)");
    let expected_path = create_test_device_path();

    let proto = boot::open_protocol_exclusive::<DevicePathFromText>(
        boot::get_handle_for_protocol::<DevicePathFromText>().unwrap(),
    )
    .unwrap();

    assert_eq!(
        &*proto.convert_text_to_device_path(text).unwrap(),
        &*expected_path
    );
}

/// Test `DevicePathFromText::convert_text_to_device_node`.
fn test_convert_text_to_device_node() {
    let path = create_test_device_path();
    let expected_node = path.node_iter().next().unwrap();

    let proto = boot::open_protocol_exclusive::<DevicePathFromText>(
        boot::get_handle_for_protocol::<DevicePathFromText>().unwrap(),
    )
    .unwrap();

    assert_eq!(
        &*proto
            .convert_text_to_device_node(cstr16!("Ata(Primary,Master,0x1)"))
            .unwrap(),
        expected_node,
    );
}

/// Test `DevicePath::DevicePath::append_path()` and `DevicePath::DevicePath::append_node()`
fn test_device_path_append() {
    let path = create_test_device_path();
    let path2 = create_test_device_path();
    let node = path.node_iter().next().unwrap();

    assert_eq!(
        path.to_string(DisplayOnly(false), AllowShortcuts(false))
            .unwrap(),
        cstr16!("Ata(Primary,Master,0x1)/VenMsg(E0C14753-F9BE-11D2-9A0C-0090273FC14D)")
    );
    assert_eq!(
        node.to_string(DisplayOnly(false), AllowShortcuts(false))
            .unwrap(),
        cstr16!("Ata(Primary,Master,0x1)")
    );

    assert_eq!(
        path.append_path(&path2).unwrap().to_string(DisplayOnly(false), AllowShortcuts(false)).unwrap(),
        cstr16!("Ata(Primary,Master,0x1)/VenMsg(E0C14753-F9BE-11D2-9A0C-0090273FC14D)/Ata(Primary,Master,0x1)/VenMsg(E0C14753-F9BE-11D2-9A0C-0090273FC14D)")
    );
    assert_eq!(
        path.append_node(node).unwrap().to_string(DisplayOnly(false), AllowShortcuts(false)).unwrap(),
        cstr16!("Ata(Primary,Master,0x1)/VenMsg(E0C14753-F9BE-11D2-9A0C-0090273FC14D)/Ata(Primary,Master,0x1)")
    );
}
