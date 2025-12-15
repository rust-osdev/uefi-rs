// SPDX-License-Identifier: MIT OR Apache-2.0

//! PCI Bus device function and bridge enumeration.

use core::mem;

use alloc::collections::btree_map::BTreeMap;
use alloc::collections::btree_set::{self, BTreeSet};

use super::PciIoAddress;
use super::root_bridge::PciRootBridgeIo;

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
    primary_bus: u8,
    secondary_bus: u8,
    subordinate_bus: u8,
    secondary_latency_timer: u8,
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

/// Struct representing the tree structure of PCI devices.
///
/// This allows iterating over all valid PCI device addresses in a tree, as well as querying
/// the tree topology.
#[derive(Debug)]
pub struct PciTree {
    segment: u32,
    devices: BTreeSet<PciIoAddress>,
    bus_anchors: BTreeMap<u8 /* bus */, PciIoAddress>,
}
impl PciTree {
    pub(crate) const fn new(segment: u32) -> Self {
        Self {
            segment,
            devices: BTreeSet::new(),
            bus_anchors: BTreeMap::new(),
        }
    }

    pub(crate) fn should_visit_bus(&self, bus: u8) -> bool {
        !self.bus_anchors.contains_key(&bus)
    }

    pub(crate) fn push_device(&mut self, addr: PciIoAddress) {
        self.devices.insert(addr);
    }

    /// Pushes a new bridge into the topology.
    ///
    /// Returns `false` if the bus is already in the topology and `true`
    /// if the bridge was added to the topology.
    pub(crate) fn push_bridge(&mut self, addr: PciIoAddress, child_bus: u8) -> bool {
        match self.bus_anchors.contains_key(&child_bus) {
            true => false,
            false => {
                self.bus_anchors.insert(child_bus, addr);
                true
            }
        }
    }

    /// Iterate over all valid PCI device addresses in this tree structure.
    pub fn iter(&self) -> btree_set::Iter<'_, PciIoAddress> {
        self.devices.iter()
    }

    /// Get the segment number of this PCI tree.
    #[must_use]
    pub const fn segment_nr(&self) -> u32 {
        self.segment
    }

    /// Query the address of the parent PCI bridge this `addr`'s bus is subordinate to.
    #[must_use]
    pub fn parent_for(&self, addr: PciIoAddress) -> Option<PciIoAddress> {
        self.bus_anchors.get(&addr.bus).cloned()
    }

    /// Iterate over all subsequent busses below the given `addr`.
    /// This yields 0 results if `addr` doesn't point to a PCI bridge.
    pub fn child_bus_of_iter(&self, addr: PciIoAddress) -> impl Iterator<Item = u8> {
        self.bus_anchors
            .iter()
            .filter(move |&(_, parent)| *parent == addr)
            .map(|(bus, _)| bus)
            .cloned()
    }
}
impl IntoIterator for PciTree {
    type Item = PciIoAddress;
    type IntoIter = btree_set::IntoIter<PciIoAddress>;

    fn into_iter(self) -> Self::IntoIter {
        self.devices.into_iter()
    }
}
impl<'a> IntoIterator for &'a PciTree {
    type Item = &'a PciIoAddress;
    type IntoIter = btree_set::Iter<'a, PciIoAddress>;

    fn into_iter(self) -> Self::IntoIter {
        self.devices.iter()
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
    tree: &mut PciTree,
) -> uefi::Result<()> {
    if get_vendor_id(proto, addr)? == 0xFFFF {
        return Ok(()); // function doesn't exist - bail instantly
    }
    tree.push_device(addr);
    let (base_class, sub_class) = get_classes(proto, addr)?;
    let header_type = get_header_type(proto, addr)? & 0b01111111;
    if base_class == 0x6 && sub_class == 0x4 && header_type == 0x01 {
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
            // Recurse into the bus namespaces on the other side of the bridge, if we haven't visited
            // the subordinate bus through a more direct path already
            if tree.push_bridge(addr, bus) {
                visit_bus(proto, PciIoAddress::new(bus, 0, 0), tree)?;
            }
        }
    }
    Ok(())
}

fn visit_device(
    proto: &mut PciRootBridgeIo,
    addr: PciIoAddress,
    tree: &mut PciTree,
) -> uefi::Result<()> {
    if get_vendor_id(proto, addr)? == 0xFFFF {
        return Ok(()); // device doesn't exist
    }
    visit_function(proto, addr.with_function(0), tree)?;
    if get_header_type(proto, addr.with_function(0))? & 0x80 != 0 {
        // This is a multi-function device - also try the remaining functions [1;7]
        // These remaining functions can be sparsely populated - as long as function 0 exists.
        for fun in 1..=7 {
            visit_function(proto, addr.with_function(fun), tree)?;
        }
    }

    Ok(())
}

pub(crate) fn visit_bus(
    proto: &mut PciRootBridgeIo,
    addr: PciIoAddress,
    tree: &mut PciTree,
) -> uefi::Result<()> {
    // Given a valid bus entry point - simply try all possible devices addresses
    for dev in 0..32 {
        visit_device(proto, addr.with_device(dev), tree)?;
    }
    Ok(())
}
