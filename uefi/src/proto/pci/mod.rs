// SPDX-License-Identifier: MIT OR Apache-2.0

//! PCI Bus specific protocols.

use core::cmp::Ordering;

use uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolWidth;

pub mod buffer;
pub mod region;
pub mod resource;
pub mod root_bridge;

/// IO Address for PCI/register IO operations
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PciIoAddress {
    /// Register number within the PCI device.
    pub reg: u8,
    /// Function number within the PCI device.
    pub fun: u8,
    /// Device number within the PCI bus.
    pub dev: u8,
    /// Bus number in the PCI hierarchy.
    pub bus: u8,
    /// Extended register number within the PCI device.
    pub ext_reg: u32,
}

impl PciIoAddress {
    /// Create address pointing to the device identified by `bus`, `dev` and `fun` ids.
    #[must_use]
    pub const fn new(bus: u8, dev: u8, fun: u8) -> Self {
        Self {
            bus,
            dev,
            fun,
            reg: 0,
            ext_reg: 0,
        }
    }

    /// Configure the **byte**-offset of the register to access.
    #[must_use]
    pub const fn with_register(&self, reg: u8) -> Self {
        let mut addr = *self;
        addr.reg = reg;
        addr.ext_reg = 0;
        addr
    }

    /// Configure the **byte**-offset of the extended register to access.
    #[must_use]
    pub const fn with_extended_register(&self, ext_reg: u32) -> Self {
        let mut addr = *self;
        addr.reg = 0;
        addr.ext_reg = ext_reg;
        addr
    }
}

impl From<u64> for PciIoAddress {
    fn from(value: u64) -> Self {
        let raw = value.to_ne_bytes();
        Self {
            reg: raw[0],
            fun: raw[1],
            dev: raw[2],
            bus: raw[3],
            ext_reg: u32::from_ne_bytes([raw[4], raw[5], raw[6], raw[7]]),
        }
    }
}

impl From<PciIoAddress> for u64 {
    fn from(value: PciIoAddress) -> Self {
        let ereg = value.ext_reg.to_ne_bytes();
        Self::from_ne_bytes([
            value.reg, value.fun, value.dev, value.bus, ereg[0], ereg[1], ereg[2], ereg[3],
        ])
    }
}

impl PartialOrd for PciIoAddress {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PciIoAddress {
    fn cmp(&self, other: &Self) -> Ordering {
        u64::from(*self).cmp(&u64::from(*other))
    }
}

/// Trait implemented by all data types that can natively be read from a PCI device.
/// Note: Not all of them have to actually be supported by the hardware at hand.
pub trait PciIoUnit: Sized + Default + Into<u64> {}
impl PciIoUnit for u8 {}
impl PciIoUnit for u16 {}
impl PciIoUnit for u32 {}
impl PciIoUnit for u64 {}

#[allow(dead_code)]
enum PciIoMode {
    Normal,
    Fifo,
    Fill,
}

fn encode_io_mode_and_unit<U: PciIoUnit>(mode: PciIoMode) -> PciRootBridgeIoProtocolWidth {
    match (mode, size_of::<U>()) {
        (PciIoMode::Normal, 1) => PciRootBridgeIoProtocolWidth::UINT8,
        (PciIoMode::Normal, 2) => PciRootBridgeIoProtocolWidth::UINT16,
        (PciIoMode::Normal, 4) => PciRootBridgeIoProtocolWidth::UINT32,
        (PciIoMode::Normal, 8) => PciRootBridgeIoProtocolWidth::UINT64,

        (PciIoMode::Fifo, 1) => PciRootBridgeIoProtocolWidth::FIFO_UINT8,
        (PciIoMode::Fifo, 2) => PciRootBridgeIoProtocolWidth::FIFO_UINT16,
        (PciIoMode::Fifo, 4) => PciRootBridgeIoProtocolWidth::FIFO_UINT32,
        (PciIoMode::Fifo, 8) => PciRootBridgeIoProtocolWidth::FIFO_UINT64,

        (PciIoMode::Fill, 1) => PciRootBridgeIoProtocolWidth::FILL_UINT8,
        (PciIoMode::Fill, 2) => PciRootBridgeIoProtocolWidth::FILL_UINT16,
        (PciIoMode::Fill, 4) => PciRootBridgeIoProtocolWidth::FILL_UINT32,
        (PciIoMode::Fill, 8) => PciRootBridgeIoProtocolWidth::FILL_UINT64,

        _ => unreachable!("Illegal PCI IO-Mode / Unit combination"),
    }
}

#[cfg(test)]
mod tests {
    use super::PciIoAddress;
    use core::mem;

    #[test]
    fn test_pci_ioaddr_raw_conversion() {
        assert_eq!(mem::size_of::<u64>(), mem::size_of::<PciIoAddress>());
        let srcaddr = PciIoAddress {
            reg: 0x11,
            fun: 0x33,
            dev: 0x55,
            bus: 0x77,
            ext_reg: 0x99bbddff,
        };
        let rawaddr: u64 = srcaddr.into();
        let dstaddr = PciIoAddress::from(rawaddr);
        assert_eq!(rawaddr, 0x99_bb_dd_ff_7755_3311);
        assert_eq!(srcaddr, dstaddr);
    }
}
