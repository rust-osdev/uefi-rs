// SPDX-License-Identifier: MIT OR Apache-2.0

use super::get_open_protocol;
use uefi::proto::pci::io::{PciIo, PciIoProtocolAttributes};

const RED_HAT_PCI_VENDOR_ID: u16 = 0x1AF4;
const VIRTIO_RNG_DEVICE_ID: u16 = 0x1005;

pub fn test() {
    let pci_handles = uefi::boot::find_handles::<PciIo>().unwrap();
    assert!(!pci_handles.is_empty(), "No PciIo handles found");

    let mut found_virtio_rng = false;

    for pci_handle in pci_handles {
        let mut pci_io = get_open_protocol::<PciIo>(pci_handle);

        let location = pci_io.location().unwrap();
        log::info!(
            "PciIo Location: segment={}, bus={}, device={}, function={}",
            location.segment,
            location.bus,
            location.device,
            location.function
        );

        let reg0 = pci_io.pci().read_one::<u32>(0).unwrap();
        let vendor_id = (reg0 & 0xFFFF) as u16;
        let device_id = (reg0 >> 16) as u16;

        let reg2 = pci_io.pci().read_one::<u32>(8).unwrap();
        let class_code = (reg2 >> 24) as u8;
        let subclass_code = ((reg2 >> 16) & 0xFF) as u8;

        log::info!(
            "PciIo device: vendor={vendor_id:04X}, device={device_id:04X}, class={class_code:02X}, subclass={subclass_code:02X}"
        );

        let supported_attrs = pci_io.supported_attributes().unwrap();
        let current_attrs = pci_io.attributes().unwrap();
        log::info!("PciIo attributes: supported={supported_attrs:?}, current={current_attrs:?}");

        // Test querying BAR attributes
        for bar_index in 0..6 {
            if let Ok(attrs) = pci_io.bar_attributes(bar_index) {
                log::info!("PciIo BAR {bar_index} attributes: {attrs:?}");
            }
        }

        // Check Option ROM access
        let rom_size = pci_io.rom_size();
        let rom_image = pci_io.rom_image();
        if rom_size > 0 {
            assert!(rom_image.is_some());
            assert_eq!(rom_image.unwrap().len(), rom_size as usize);
        } else {
            assert!(rom_image.is_none());
        }

        // Test flush
        pci_io.flush().unwrap();

        if vendor_id == RED_HAT_PCI_VENDOR_ID && device_id == VIRTIO_RNG_DEVICE_ID {
            found_virtio_rng = true;

            // Test BAR0 IO access for virtio RNG
            let device_features = pci_io.io(0).read_one::<u32>(0).unwrap();
            log::info!("Virtio RNG device features via PciIo: {device_features:#0b}");
        }

        // Test setting attributes. Skip the virtio RNG device: OVMF's own VirtioRngDxe driver
        // backs `EFI_RNG_PROTOCOL` with this same device, and momentarily clearing its
        // bus-master/IO/memory decode here desyncs that driver's virtqueue, hanging the later
        // `rng::test()` call to `GetRng` forever.
        if !(vendor_id == RED_HAT_PCI_VENDOR_ID && device_id == VIRTIO_RNG_DEVICE_ID) {
            unsafe {
                pci_io
                    .set_attributes(PciIoProtocolAttributes::empty())
                    .unwrap();
                pci_io.set_attributes(current_attrs).unwrap();
            }
        }
    }

    assert!(found_virtio_rng, "Virtio RNG device not found via PciIo");
}
