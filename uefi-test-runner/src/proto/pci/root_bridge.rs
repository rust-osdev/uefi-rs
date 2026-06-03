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
use uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolAttributes;

const RED_HAT_PCI_VENDOR_ID: u16 = 0x1AF4;
const VIRTIO_RNG_DEVICE_ID: u16 = 0x1005;
const MASS_STORAGE_CTRL_CLASS_CODE: u8 = 0x1;
const SATA_CTRL_SUBCLASS_CODE: u8 = 0x6;

const REG_SIZE: u8 = size_of::<u32>() as u8;

pub fn test() {
    test_enumeration_and_address_space_access();
    test_attributes();
}

fn test_enumeration_and_address_space_access() {
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
            let reg2 = pci_proto
                .pci()
                .read_one::<u32>(addr.with_register(2 * REG_SIZE))
                .unwrap();
            let reg3 = pci_proto
                .pci()
                .read_one::<u32>(addr.with_register(3 * REG_SIZE))
                .unwrap();

            let vendor_id = (reg0 & 0xFFFF) as u16;
            let device_id = (reg0 >> 16) as u16;
            let class_code = (reg2 >> 24) as u8;
            let subclass_code = ((reg2 >> 16) & 0xFF) as u8;
            let header_type = ((reg3 >> 16) & 0x7F) as u8;
            let device_path = pci_tree.device_path(&root_device_path, addr).unwrap();
            let device_path_str = device_path
                .to_string16(DisplayOnly(false), AllowShortcuts(false))
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

            if vendor_id == RED_HAT_PCI_VENDOR_ID && device_id == VIRTIO_RNG_DEVICE_ID {
                assert_eq!(
                    header_type, 0x00,
                    "unexpected header type for PCI virtio rng device"
                );

                let mut bars = [0; 6];
                pci_proto
                    .pci()
                    .read::<u32>(addr.with_register(4 * REG_SIZE), &mut bars)
                    .unwrap();
                log::info!("BARS: {bars:#x?}");

                let mut bars = bars.into_iter();
                let mut next_bar = bars.next();
                while let Some(bar) = next_bar {
                    next_bar = bars.next();

                    let bar = decode_bar(bar, next_bar);
                    match bar {
                        Bar::Io(base) => {
                            // Virtio RNG devices have a device features register that is safe to
                            // read at the start of the I/O space.
                            let device_features =
                                pci_proto.io().read_one::<u32>(u64::from(base)).unwrap();
                            log::info!("Device Features: {device_features:#0b}");
                        }
                        Bar::Mem32 {
                            base,
                            prefetchable: true,
                        } => {
                            // Reading from a prefetchable MMIO region is always non-destructive.
                            pci_proto.memory().read_one::<u32>(u64::from(base)).unwrap();
                        }
                        Bar::Mem64 {
                            base,
                            prefetchable: true,
                        } => {
                            // Reading from a prefetchable MMIO region is always non-destructive.
                            pci_proto.memory().read_one::<u32>(base).unwrap();
                        }
                        _ => {}
                    }

                    log::info!("BAR: {bar:x?}");
                    if let Bar::Mem64 {
                        base: _,
                        prefetchable: _,
                    } = bar
                    {
                        next_bar = bars.next();
                    }
                }
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
            .to_string16(DisplayOnly(false), AllowShortcuts(false))
            .unwrap()
            .to_string();
        assert!(mass_storage_dev_paths.contains(&device_path));
    }
}

fn test_attributes() {
    let pci_handles = uefi::boot::find_handles::<PciRootBridgeIo>().unwrap();
    for pci_handle in pci_handles {
        let mut pci_proto = get_open_protocol::<PciRootBridgeIo>(pci_handle);

        let supported_attributes = pci_proto.supported_attributes().unwrap();
        log::info!("Supported Attributes: {supported_attributes:?}");

        let current_attributes = pci_proto.attributes().unwrap();
        log::info!("Current Attributes: {current_attributes:?}");

        unsafe {
            pci_proto
                .set_attributes(PciRootBridgeIoProtocolAttributes::empty())
                .unwrap()
        }
        unsafe { pci_proto.set_attributes(supported_attributes).unwrap() }
        unsafe { pci_proto.set_attributes(current_attributes).unwrap() }
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

fn decode_bar(bar: u32, next_bar: Option<u32>) -> Bar {
    if bar & 0b1 == 0b0 {
        match (bar & 0b110) >> 1 {
            0b00 => Bar::Mem32 {
                base: bar & !0b1111,
                prefetchable: bar & 0b1000 != 0,
            },
            0b10 => {
                if let Some(next_bar) = next_bar {
                    Bar::Mem64 {
                        base: u64::from(bar & !0b1111) | (u64::from(next_bar) << 32),
                        prefetchable: bar & 0b1000 != 0,
                    }
                } else {
                    unreachable!("PCI hardware error")
                }
            }
            _ => unimplemented!(),
        }
    } else {
        Bar::Io(bar & !0b11)
    }
}

#[derive(Debug)]
enum Bar {
    Mem32 { base: u32, prefetchable: bool },
    Mem64 { base: u64, prefetchable: bool },
    Io(u32),
}
