// SPDX-License-Identifier: MIT OR Apache-2.0

//! PCI Bus specific protocols.

use core::cmp::Ordering;

use uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolWidth;

pub mod configuration;
#[cfg(feature = "alloc")]
pub mod enumeration;
pub mod page;
pub mod region;
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

    /// Construct a new address with the bus address set to the given value
    #[must_use]
    pub const fn with_bus(&self, bus: u8) -> Self {
        let mut addr = *self;
        addr.bus = bus;
        addr
    }

    /// Construct a new address with the device address set to the given value
    #[must_use]
    pub const fn with_device(&self, dev: u8) -> Self {
        let mut addr = *self;
        addr.dev = dev;
        addr
    }

    /// Construct a new address with the function address set to the given value
    #[must_use]
    pub const fn with_function(&self, fun: u8) -> Self {
        let mut addr = *self;
        addr.fun = fun;
        addr
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
    fn cmp(&self, o: &Self) -> Ordering {
        // extract fields because taking references to unaligned fields in packed structs is a nono
        let (bus, dev, fun, reg, ext_reg) = (self.bus, self.dev, self.fun, self.reg, self.ext_reg);
        let (o_bus, o_dev, o_fun, o_reg, o_ext_reg) = (o.bus, o.dev, o.fun, o.reg, o.ext_reg);
        bus.cmp(&o_bus)
            .then(dev.cmp(&o_dev))
            .then(fun.cmp(&o_fun))
            .then(reg.cmp(&o_reg))
            .then(ext_reg.cmp(&o_ext_reg))
    }
}

// ############################################################################################

/// Trait implemented by all data types that can natively be read from a PCI device.
/// Note: Not all of them have to actually be supported by the hardware at hand.
pub trait PciIoUnit: Sized + Default {}
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
    use core::cmp::Ordering;

    use super::PciIoAddress;

    #[test]
    #[allow(clippy::unusual_byte_groupings)]
    fn test_pci_ioaddr_raw_conversion() {
        assert_eq!(size_of::<u64>(), size_of::<PciIoAddress>());
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

    #[test]
    fn test_pci_order() {
        let addr0_0_0 = PciIoAddress::new(0, 0, 0);
        let addr0_0_1 = PciIoAddress::new(0, 0, 1);
        let addr0_1_0 = PciIoAddress::new(0, 1, 0);
        let addr1_0_0 = PciIoAddress::new(1, 0, 0);

        assert_eq!(addr0_0_0.cmp(&addr0_0_0), Ordering::Equal);
        assert_eq!(addr0_0_0.cmp(&addr0_0_1), Ordering::Less);
        assert_eq!(addr0_0_0.cmp(&addr0_1_0), Ordering::Less);
        assert_eq!(addr0_0_0.cmp(&addr1_0_0), Ordering::Less);

        assert_eq!(addr0_0_1.cmp(&addr0_0_0), Ordering::Greater);
        assert_eq!(addr0_0_1.cmp(&addr0_0_1), Ordering::Equal);
        assert_eq!(addr0_0_1.cmp(&addr0_1_0), Ordering::Less);
        assert_eq!(addr0_0_1.cmp(&addr1_0_0), Ordering::Less);

        assert_eq!(addr0_1_0.cmp(&addr0_0_0), Ordering::Greater);
        assert_eq!(addr0_1_0.cmp(&addr0_0_1), Ordering::Greater);
        assert_eq!(addr0_1_0.cmp(&addr0_1_0), Ordering::Equal);
        assert_eq!(addr0_1_0.cmp(&addr1_0_0), Ordering::Less);

        assert_eq!(addr1_0_0.cmp(&addr0_0_0), Ordering::Greater);
        assert_eq!(addr1_0_0.cmp(&addr0_0_1), Ordering::Greater);
        assert_eq!(addr1_0_0.cmp(&addr0_1_0), Ordering::Greater);
        assert_eq!(addr1_0_0.cmp(&addr1_0_0), Ordering::Equal);

        assert_eq!(addr0_0_0.cmp(&addr0_0_0.with_register(1)), Ordering::Less);
        assert_eq!(
            addr0_0_0.with_register(1).cmp(&addr0_0_0),
            Ordering::Greater
        );
        assert_eq!(
            addr0_0_0.cmp(&addr0_0_0.with_extended_register(1)),
            Ordering::Less
        );
        assert_eq!(
            addr0_0_0.with_extended_register(1).cmp(&addr0_0_0),
            Ordering::Greater
        );
    }
}
