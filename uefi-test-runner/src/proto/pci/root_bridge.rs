// SPDX-License-Identifier: MIT OR Apache-2.0

use core::ptr;
use uefi::boot::{OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol, image_handle};
use uefi::proto::ProtocolPointer;
use uefi::proto::pci::PciIoAddress;
use uefi::proto::pci::root_bridge::{AttributeReport, PciRootBridgeIo};
use uefi::Handle;
use uefi_raw::protocol::pci::resource::QWordAddressSpaceDescriptor;
use uefi_raw::protocol::pci::root_bridge::{
    PciRootBridgeIoProtocolAttribute, PciRootBridgeIoProtocolOperation,
};
use uefi_raw::table::boot::MemoryType;

const RED_HAT_PCI_VENDOR_ID: u16 = 0x1AF4;
const MASS_STORAGE_CTRL_CLASS_CODE: u8 = 0x1;
const SATA_CTRL_SUBCLASS_CODE: u8 = 0x6;
const DEVICE_IVSHMEM: u16 = 0x1110;

const REG_SIZE: u8 = size_of::<u32>() as u8;

pub fn test_io() {
    let pci_handles = uefi::boot::find_handles::<PciRootBridgeIo>().unwrap();

    let mut red_hat_dev_cnt = 0;
    let mut mass_storage_ctrl_cnt = 0;
    let mut sata_ctrl_cnt = 0;

    for pci_handle in pci_handles {
        let pci_proto = get_open_protocol::<PciRootBridgeIo>(pci_handle);
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
                        "PCI Device: [{bus}, {dev}, {fun}]: vendor={vendor_id:04X}, device={device_id:04X}, class={class_code:02X}, subclass={subclass_code:02X}"
                    );
                }
            }
        }
    }

    assert!(red_hat_dev_cnt > 0);
    assert!(mass_storage_ctrl_cnt > 0);
    assert!(sata_ctrl_cnt > 0);
}

pub fn test_buffer() {
    let pci_handles = uefi::boot::find_handles::<PciRootBridgeIo>().unwrap();

    for pci_handle in pci_handles {
        let pci_proto = get_open_protocol::<PciRootBridgeIo>(pci_handle);
        let mut buffer = pci_proto
            .allocate_buffer::<[u8; 4096]>(
                MemoryType::BOOT_SERVICES_DATA,
                None,
                PciRootBridgeIoProtocolAttribute::MEMORY_WRITE_COMBINE,
            )
            .unwrap();

        let buffer = unsafe {
            buffer.assume_init_mut().fill(0);
            buffer.assume_init()
        };

        assert_eq!(buffer.as_ptr().addr() % 4096, 0);
    }
}

pub fn test_mapping() {
    let pci_handles = uefi::boot::find_handles::<PciRootBridgeIo>().unwrap();

    for pci_handle in pci_handles {
        let pci_proto = get_open_protocol::<PciRootBridgeIo>(pci_handle);

        let mut buffer = pci_proto
            .allocate_buffer::<[u8; 4096]>(
                MemoryType::BOOT_SERVICES_DATA,
                None,
                PciRootBridgeIoProtocolAttribute::MEMORY_WRITE_COMBINE,
            )
            .unwrap();
        let buffer = unsafe {
            buffer.assume_init_mut().fill(0);
            buffer.assume_init()
        };

        let mapped = pci_proto
            .map(
                PciRootBridgeIoProtocolOperation::BUS_MASTER_COMMON_BUFFER64,
                buffer.as_ref(),
            )
            .unwrap();
        if mapped.region().device_address == buffer.as_ptr().addr() as u64 {
            info!("This PCI device uses identity mapping");
        } else {
            info!("This PCI device uses different mapping from CPU");
        }
    }
}

pub fn test_copy() {
    let pci_handles = uefi::boot::find_handles::<PciRootBridgeIo>().unwrap();

    for pci_handle in pci_handles {
        let pci_proto = get_open_protocol::<PciRootBridgeIo>(pci_handle);
        for bus in 0..=255 {
            for dev in 0..32 {
                for fun in 0..8 {
                    let addr = PciIoAddress::new(bus, dev, fun);
                    let pci_access = pci_proto.pci();
                    let Ok(reg0) = pci_access.read_one::<u32>(addr.with_register(0)) else {
                        continue;
                    };
                    if reg0 == 0xFFFFFFFF {
                        continue; // not a valid device
                    }

                    let vendor_id = (reg0 & 0xFFFF) as u16;
                    let device_id = (reg0 >> 16) as u16;

                    if vendor_id != RED_HAT_PCI_VENDOR_ID {
                        continue;
                    }
                    if device_id != DEVICE_IVSHMEM {
                        continue;
                    }

                    let header_type: u8 = pci_access.read_one(addr.with_register(0xE)).unwrap();
                    assert_eq!(header_type, 0);

                    let command_value = pci_access.read_one::<u16>(addr.with_register(4)).unwrap();
                    pci_access
                        .write_one::<u16>(addr.with_register(4), command_value & !0x11)
                        .unwrap();

                    let bar2 = pci_access
                        .read_one::<u64>(addr.with_register(0x18))
                        .unwrap(); // reads both bar2 and bar3 since it's 64bit
                    assert_eq!(bar2 & 0b1, 0);
                    assert_eq!((bar2 & 0b110) >> 1, 2); // make sure it's actually 64bit

                    let bar2_value = bar2 & 0xFFFFFFFFFFFFFFF0;
                    let bar2_size = {
                        pci_access
                            .write_one(addr.with_register(0x18), u32::MAX)
                            .unwrap();
                        let value: u32 = pci_access.read_one(addr.with_register(0x18)).unwrap();
                        let size = (!value).wrapping_add(1);
                        pci_access
                            .write_one(addr.with_register(0x18), bar2 as u32)
                            .unwrap();
                        size
                    };
                    assert!(bar2_size >= 0x1000 * 2);

                    pci_access
                        .write_one::<u16>(addr.with_register(4), command_value | 0b10)
                        .unwrap();

                    let (src, dst) = unsafe {
                        let src = ptr::slice_from_raw_parts_mut(
                            (bar2_value as usize) as *mut u32,
                            0x1000 / size_of::<u32>(),
                        )
                        .as_mut()
                        .unwrap();
                        let dst = ptr::slice_from_raw_parts(
                            (bar2_value as usize + 0x1000) as *mut u32,
                            0x1000 / size_of::<u32>(),
                        )
                        .as_ref()
                        .unwrap();
                        (src, dst)
                    };
                    src.fill(0xDEADBEEF);

                    pci_proto.copy::<u32>(dst, src).unwrap();

                    assert!(dst.iter().all(|&b| b == 0xDEADBEEF));
                    break;
                }
            }
        }
    }
}

pub fn test_config() {
    let pci_handles = uefi::boot::find_handles::<PciRootBridgeIo>().unwrap();

    for pci_handle in pci_handles {
        let pci_proto = get_open_protocol::<PciRootBridgeIo>(pci_handle);
        let Ok(configuration) = pci_proto.configuration() else {
            continue;
        };
        info!("Found {} configurations", configuration.len());
    }
}

pub fn test_attributes() {
    let pci_handles = uefi::boot::find_handles::<PciRootBridgeIo>().unwrap();

    for pci_handle in pci_handles {
        let pci_proto = get_open_protocol::<PciRootBridgeIo>(pci_handle);
        let AttributeReport { supported, .. } = pci_proto.get_attributes();

        pci_proto
            .set_attributes(PciRootBridgeIoProtocolAttribute::empty(), None)
            .unwrap();
        pci_proto.set_attributes(supported, None).unwrap();
    }
}

pub fn test_sizes() {
    assert_eq!(size_of::<QWordAddressSpaceDescriptor>(), 0x2E);
    assert_eq!(size_of::<PciIoAddress>(), size_of::<u64>());
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
