// SPDX-License-Identifier: MIT OR Apache-2.0

use core::mem;
use uefi::Handle;
use uefi::boot::{OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol, image_handle};
use uefi::proto::ProtocolPointer;
use uefi::proto::pci::PciIoAddress;
use uefi::proto::pci::root_bridge::PciRootBridgeIo;

const RED_HAT_PCI_VENDOR_ID: u16 = 0x1AF4;
const MASS_STORAGE_CTRL_CLASS_CODE: u8 = 0x1;
const SATA_CTRL_SUBCLASS_CODE: u8 = 0x6;

const REG_SIZE: u8 = mem::size_of::<u32>() as u8;

pub fn test() {
    let pci_handles = uefi::boot::find_handles::<PciRootBridgeIo>().unwrap();

    let mut red_hat_dev_cnt = 0;
    let mut mass_storage_ctrl_cnt = 0;
    let mut sata_ctrl_cnt = 0;

    for pci_handle in pci_handles {
        let mut pci_proto = get_open_protocol::<PciRootBridgeIo>(pci_handle);

        for bus in 0..=255 {
            for dev in 0..32 {
                for fun in 0..8 {
                    let addr = PciIoAddress::new(bus, dev, fun);
                    let Ok(reg0) = pci_proto.pci().read_one::<u32>(addr.with_register(0)) else {
                        continue;
                    };
                    if reg0 == 0xFFFFFFFF {
                        continue; // not a valid device
                    }
                    let reg1 = pci_proto
                        .pci()
                        .read_one::<u32>(addr.with_register(2 * REG_SIZE))
                        .unwrap();

                    let vendor_id = (reg0 & 0xFFFF) as u16;
                    let device_id = (reg0 >> 16) as u16;
                    if vendor_id == RED_HAT_PCI_VENDOR_ID {
                        red_hat_dev_cnt += 1;
                    }

                    let class_code = (reg1 >> 24) as u8;
                    let subclass_code = ((reg1 >> 16) & 0xFF) as u8;
                    if class_code == MASS_STORAGE_CTRL_CLASS_CODE {
                        mass_storage_ctrl_cnt += 1;

                        if subclass_code == SATA_CTRL_SUBCLASS_CODE {
                            sata_ctrl_cnt += 1;
                        }
                    }

                    log::info!(
                        "PCI Device: [{}, {}, {}]: vendor={:04X}, device={:04X}, class={:02X}, subclass={:02X}",
                        bus,
                        dev,
                        fun,
                        vendor_id,
                        device_id,
                        class_code,
                        subclass_code
                    );
                }
            }
        }
    }

    assert!(red_hat_dev_cnt > 0);
    assert!(mass_storage_ctrl_cnt > 0);
    assert!(sata_ctrl_cnt > 0);
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
