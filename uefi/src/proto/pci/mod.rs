// SPDX-License-Identifier: MIT OR Apache-2.0

//! PCI Bus specific protocols.

use uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolWidth;

pub mod root_bridge;

/// IO Address for PCI/register IO operations
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl From<PciIoAddress> for u64 {
    fn from(value: PciIoAddress) -> Self {
        unsafe { core::mem::transmute(value) }
    }
}

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
    match (mode, core::mem::size_of::<U>()) {
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
