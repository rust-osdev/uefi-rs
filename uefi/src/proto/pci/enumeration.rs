// SPDX-License-Identifier: MIT OR Apache-2.0

//! PCI Bus device function and bridge enumeration.

use core::mem;

use alloc::collections::btree_set::BTreeSet;

use super::root_bridge::PciRootBridgeIo;
use super::{FullPciIoAddress, PciIoAddress};

#[allow(unused)]
#[derive(Clone, Copy, Debug)]
struct PciRegister0 {
    vendor_id: u16,
    device_id: u16,
}

#[allow(unused)]
#[derive(Clone, Copy, Debug)]
struct PciRegister2 {
    revision_id: u8,
    prog_if: u8,
    subclass: u8,
    class: u8,
}

#[allow(unused)]
#[derive(Clone, Copy, Debug)]
struct PciRegister3 {
    cache_line_size: u8,
    latency_timer: u8,
    header_type: u8,
    bist: u8,
}

#[allow(unused)]
#[derive(Clone, Copy, Debug)]
struct PciHeader1Register6 {
    secondary_latency_timer: u8,
    subordinate_bus: u8,
    secondary_bus: u8,
    primary_bus: u8,
}

/// Read the 4byte pci register with the given `addr` and cast it into the given structured representation.
fn read_device_register_u32<T: Sized + Copy>(
    proto: &mut PciRootBridgeIo,
    addr: PciIoAddress,
) -> uefi::Result<T> {
    unsafe {
        let raw = proto.pci().read_one::<u32>(addr)?;
        let reg: T = mem::transmute_copy(&raw);
        Ok(reg)
    }
}

// ##########################################################################################
// # Query Helpers (read from a device's configuration registers)

fn get_vendor_id(proto: &mut PciRootBridgeIo, addr: PciIoAddress) -> uefi::Result<u16> {
    read_device_register_u32::<PciRegister0>(proto, addr.with_register(0)).map(|v| v.vendor_id)
}

fn get_classes(proto: &mut PciRootBridgeIo, addr: PciIoAddress) -> uefi::Result<(u8, u8)> {
    let reg = read_device_register_u32::<PciRegister2>(proto, addr.with_register(2 * 4))?;
    Ok((reg.class, reg.subclass))
}

fn get_header_type(proto: &mut PciRootBridgeIo, addr: PciIoAddress) -> uefi::Result<u8> {
    read_device_register_u32::<PciRegister3>(proto, addr.with_register(3 * 4))
        .map(|v| v.header_type)
}

fn get_secondary_bus_range(
    proto: &mut PciRootBridgeIo,
    addr: PciIoAddress,
) -> uefi::Result<(u8, u8)> {
    let reg = read_device_register_u32::<PciHeader1Register6>(proto, addr.with_register(6 * 4))?;
    Ok((reg.secondary_bus, reg.subordinate_bus))
}

// ##########################################################################################
// # Recursive visitor implementation

fn visit_function(
    proto: &mut PciRootBridgeIo,
    addr: PciIoAddress,
    queue: &mut BTreeSet<FullPciIoAddress>,
) -> uefi::Result<()> {
    if get_vendor_id(proto, addr)? == 0xFFFF {
        return Ok(()); // function doesn't exist - bail instantly
    }
    queue.insert(FullPciIoAddress::new(proto.segment_nr(), addr));
    let (base_class, sub_class) = get_classes(proto, addr)?;
    if base_class == 0x6 && sub_class == 0x4 && get_header_type(proto, addr)? == 0x01 {
        // This is a PCI-to-PCI bridge controller. The current `addr` is the address with which it's
        // mounted in the PCI tree we are currently traversing. Now we query its header, where
        // the bridge tells us a range of addresses [secondary;subordinate], with which the other
        // side of the bridge is mounted into the PCI tree.
        let (secondary_bus_nr, subordinate_bus_nr) = get_secondary_bus_range(proto, addr)?;
        if secondary_bus_nr == 0 || subordinate_bus_nr < secondary_bus_nr {
            // If the secondary bus number is the root number, or if the range is invalid - this hardware
            // is so horribly broken that we refrain from touching it. It might explode - or worse!
            return Ok(());
        }
        for bus in secondary_bus_nr..=subordinate_bus_nr {
            // Recurse into the bus namespaces on the other side of the bridge
            visit_bus(proto, PciIoAddress::new(bus, 0, 0), queue)?;
        }
    }
    Ok(())
}

fn visit_device(
    proto: &mut PciRootBridgeIo,
    addr: PciIoAddress,
    queue: &mut BTreeSet<FullPciIoAddress>,
) -> uefi::Result<()> {
    if get_vendor_id(proto, addr)? == 0xFFFF {
        return Ok(()); // device doesn't exist
    }
    visit_function(proto, addr.with_function(0), queue)?;
    if get_header_type(proto, addr.with_function(0))? & 0x80 != 0 {
        // This is a multi-function device - also try the remaining functions [1;7]
        // These remaining functions can be sparsely populated - as long as function 0 exists.
        for fun in 1..=7 {
            visit_function(proto, addr.with_function(fun), queue)?;
        }
    }

    Ok(())
}

pub(crate) fn visit_bus(
    proto: &mut PciRootBridgeIo,
    addr: PciIoAddress,
    queue: &mut BTreeSet<FullPciIoAddress>,
) -> uefi::Result<()> {
    // Given a valid bus entry point - simply try all possible devices addresses
    for dev in 0..32 {
        visit_device(proto, addr.with_device(dev), queue)?;
    }
    Ok(())
}
