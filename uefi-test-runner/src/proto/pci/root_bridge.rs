// SPDX-License-Identifier: MIT OR Apache-2.0

use alloc::collections::btree_set::BTreeSet;
use alloc::string::ToString;
use uefi::Handle;
use uefi::boot::{OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol, image_handle};
use uefi::proto::ProtocolPointer;
use uefi::proto::device_path::DevicePath;
use uefi::proto::device_path::text::{AllowShortcuts, DisplayOnly};
use uefi::proto::pci::root_bridge::PciRootBridgeIo;
use uefi::proto::scsi::pass_thru::ExtScsiPassThru;

const RED_HAT_PCI_VENDOR_ID: u16 = 0x1AF4;
const MASS_STORAGE_CTRL_CLASS_CODE: u8 = 0x1;
const SATA_CTRL_SUBCLASS_CODE: u8 = 0x6;

const REG_SIZE: u8 = size_of::<u32>() as u8;

pub fn test() {
    let pci_handles = uefi::boot::find_handles::<PciRootBridgeIo>().unwrap();

    let mut sata_ctrl_cnt = 0;
    let mut red_hat_dev_cnt = 0;
    let mut mass_storage_ctrl_cnt = 0;
    let mut mass_storage_dev_paths = BTreeSet::new();

    for pci_handle in pci_handles {
        let mut pci_proto = get_open_protocol::<PciRootBridgeIo>(pci_handle);
        let root_device_path = get_open_protocol::<DevicePath>(pci_handle);

        let pci_tree = pci_proto.enumerate().unwrap();
        for addr in pci_tree.iter().cloned() {
            let reg0 = pci_proto
                .pci()
                .read_one::<u32>(addr.with_register(0))
                .unwrap();
            let reg1 = pci_proto
                .pci()
                .read_one::<u32>(addr.with_register(2 * REG_SIZE))
                .unwrap();

            let vendor_id = (reg0 & 0xFFFF) as u16;
            let device_id = (reg0 >> 16) as u16;
            let class_code = (reg1 >> 24) as u8;
            let subclass_code = ((reg1 >> 16) & 0xFF) as u8;
            let device_path = pci_tree.device_path(&root_device_path, addr).unwrap();
            let device_path_str = device_path
                .to_string(DisplayOnly(false), AllowShortcuts(false))
                .unwrap()
                .to_string();

            if vendor_id == RED_HAT_PCI_VENDOR_ID {
                red_hat_dev_cnt += 1;
            }
            if class_code == MASS_STORAGE_CTRL_CLASS_CODE {
                mass_storage_ctrl_cnt += 1;
                if subclass_code == SATA_CTRL_SUBCLASS_CODE {
                    sata_ctrl_cnt += 1;
                }
                mass_storage_dev_paths.insert(device_path_str.clone());
            }

            let (bus, dev, fun) = (addr.bus, addr.dev, addr.fun);
            log::info!(
                "PCI Device: [{bus:02x}, {dev:02x}, {fun:02x}]: vendor={vendor_id:04X}, device={device_id:04X}, class={class_code:02X}, subclass={subclass_code:02X} - {}",
                device_path_str
            );
            for child_bus in pci_tree.child_bus_of_iter(addr) {
                log::info!(" \\- Bus: {child_bus:02x}");
            }
        }
    }

    assert!(sata_ctrl_cnt > 0);
    assert!(red_hat_dev_cnt > 0);
    assert!(mass_storage_ctrl_cnt > 0);
    assert_eq!(mass_storage_ctrl_cnt, mass_storage_dev_paths.len());

    // Check that all `ExtScsiPassThru` instances' device paths have been found
    let scsi_handles = uefi::boot::find_handles::<ExtScsiPassThru>().unwrap();
    for scsi_handle in scsi_handles {
        let device_path = get_open_protocol::<DevicePath>(scsi_handle);
        let device_path = device_path
            .to_string(DisplayOnly(false), AllowShortcuts(false))
            .unwrap()
            .to_string();
        assert!(mass_storage_dev_paths.contains(&device_path));
    }
}

fn get_open_protocol<P: ProtocolPointer + ?Sized>(handle: Handle) -> ScopedProtocol<P> {
    let open_opts = OpenProtocolParams {
        handle,
        agent: image_handle(),
        controller: None,
    };
    let open_attrs = OpenProtocolAttributes::GetProtocol;
    unsafe { uefi::boot::open_protocol(open_opts, open_attrs).unwrap() }
}
