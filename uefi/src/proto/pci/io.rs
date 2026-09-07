// SPDX-License-Identifier: MIT OR Apache-2.0

//! PCI I/O Protocol.

use core::ptr;
use core::time::Duration;

use super::{PciIoMode, PciIoUnit, encode_pci_io_mode_and_unit};
use crate::proto::pci::root_bridge::{PciIoSpace, PciMemorySpace};
use crate::{Result, Status, StatusExt};

use uefi_macros::unsafe_protocol;
use uefi_raw::protocol::pci::io::PciIoProtocol;

pub use uefi_raw::protocol::pci::io::{
    PCI_IO_PASS_THROUGH_BAR, PciIoProtocolAccess, PciIoProtocolAttributeOperation,
    PciIoProtocolAttributes, PciIoProtocolConfigAccess, PciIoProtocolOperation, PciIoProtocolWidth,
};

/// Location of a PCI controller in the PCI hierarchy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PciLocation {
    /// PCI segment number.
    pub segment: usize,
    /// PCI bus number.
    pub bus: usize,
    /// PCI device number.
    pub device: usize,
    /// PCI function number.
    pub function: usize,
}

/// Protocol that provides access to a PCI controller.
///
/// # UEFI Specification
/// Provides the basic Memory, I/O, PCI configuration, and DMA interfaces that are
/// used to abstract accesses to PCI controllers.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(PciIoProtocol::GUID)]
pub struct PciIo(PciIoProtocol);

impl PciIo {
    /// Retrieves this PCI controller's current PCI segment number, bus number,
    /// device number, and function number.
    pub fn location(&self) -> Result<PciLocation> {
        let mut segment = 0;
        let mut bus = 0;
        let mut device = 0;
        let mut function = 0;

        // SAFETY: The memory is valid.
        unsafe {
            (self.0.get_location)(&self.0, &mut segment, &mut bus, &mut device, &mut function)
                .to_result_with_val(|| PciLocation {
                    segment,
                    bus,
                    device,
                    function,
                })
        }
    }

    /// Access PCI controller registers in the configuration space on this device.
    pub const fn pci(&mut self) -> PciIoConfigAccess<'_> {
        PciIoConfigAccess {
            proto: &mut self.0,
            access: &mut self.0.pci,
        }
    }

    /// Access PCI controller registers in the memory space for the given BAR.
    ///
    /// Pass [`PCI_IO_PASS_THROUGH_BAR`] to bypass BAR-relative addressing.
    pub const fn memory(&mut self, bar_index: u8) -> PciIoBarAccess<'_, PciMemorySpace> {
        PciIoBarAccess {
            proto: &mut self.0,
            access: &mut self.0.mem,
            bar_index,
            _address_space: PciMemorySpace,
        }
    }

    /// Access PCI controller registers in the I/O space for the given BAR.
    ///
    /// Pass [`PCI_IO_PASS_THROUGH_BAR`] to bypass BAR-relative addressing.
    pub const fn io(&mut self, bar_index: u8) -> PciIoBarAccess<'_, PciIoSpace> {
        PciIoBarAccess {
            proto: &mut self.0,
            access: &mut self.0.io,
            bar_index,
            _address_space: PciIoSpace,
        }
    }

    /// Reads from the memory space of a PCI controller. Returns either when the polling exit
    /// criteria is satisfied or after `delay` has elapsed.
    pub fn poll_mem<U: PciIoUnit>(
        &mut self,
        bar_index: u8,
        offset: u64,
        mask: u64,
        value: u64,
        delay: Duration,
    ) -> Result<u64> {
        let width_mode = encode_pci_io_mode_and_unit::<U>(PciIoMode::Normal);
        let delay = (delay.as_nanos() / 100) as u64;
        let mut result = 0;
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.poll_mem)(
                &mut self.0,
                width_mode,
                bar_index,
                offset,
                mask,
                value,
                delay,
                &mut result,
            )
            .to_result_with_val(|| result)
        }
    }

    /// Reads from the I/O space of a PCI controller. Returns either when the polling exit
    /// criteria is satisfied or after `delay` has elapsed.
    pub fn poll_io<U: PciIoUnit>(
        &mut self,
        bar_index: u8,
        offset: u64,
        mask: u64,
        value: u64,
        delay: Duration,
    ) -> Result<u64> {
        let width_mode = encode_pci_io_mode_and_unit::<U>(PciIoMode::Normal);
        let delay = (delay.as_nanos() / 100) as u64;
        let mut result = 0;
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.poll_io)(
                &mut self.0,
                width_mode,
                bar_index,
                offset,
                mask,
                value,
                delay,
                &mut result,
            )
            .to_result_with_val(|| result)
        }
    }

    /// Enables a PCI driver to copy one region of PCI memory space to another region of PCI
    /// memory space.
    pub fn copy_mem<U: PciIoUnit>(
        &mut self,
        dest_bar_index: u8,
        dest_offset: u64,
        src_bar_index: u8,
        src_offset: u64,
        count: usize,
    ) -> Result<()> {
        let width_mode = encode_pci_io_mode_and_unit::<U>(PciIoMode::Normal);
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.copy_mem)(
                &mut self.0,
                width_mode,
                dest_bar_index,
                dest_offset,
                src_bar_index,
                src_offset,
                count,
            )
            .to_result()
        }
    }

    /// Flushes all PCI posted write transactions from a PCI host bridge to system memory.
    pub fn flush(&mut self) -> Result<()> {
        // SAFETY: The memory is valid.
        unsafe { (self.0.flush)(&mut self.0).to_result() }
    }

    /// Returns the set of [`PciIoProtocolAttributes`] that this PCI controller supports.
    pub fn supported_attributes(&mut self) -> Result<PciIoProtocolAttributes> {
        let mut result = 0;
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.attributes)(
                &mut self.0,
                PciIoProtocolAttributeOperation::SUPPORTED,
                0,
                &mut result,
            )
            .to_result_with_val(|| PciIoProtocolAttributes::from_bits_retain(result))
        }
    }

    /// Returns the [`PciIoProtocolAttributes`] that this PCI controller is currently using.
    pub fn attributes(&mut self) -> Result<PciIoProtocolAttributes> {
        let mut result = 0;
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.attributes)(
                &mut self.0,
                PciIoProtocolAttributeOperation::GET,
                0,
                &mut result,
            )
            .to_result_with_val(|| PciIoProtocolAttributes::from_bits_retain(result))
        }
    }

    /// Sets [`PciIoProtocolAttributes`] for this PCI controller.
    ///
    /// # Safety
    ///
    /// The new [`PciIoProtocolAttributes`] must be valid for the current system configuration.
    pub unsafe fn set_attributes(&mut self, attributes: PciIoProtocolAttributes) -> Result {
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.attributes)(
                &mut self.0,
                PciIoProtocolAttributeOperation::SET,
                attributes.bits(),
                ptr::null_mut(),
            )
            .to_result()
        }
    }

    /// Enables [`PciIoProtocolAttributes`] for this PCI controller.
    ///
    /// # Safety
    ///
    /// Enabling these attributes must be safe for the current system configuration.
    pub unsafe fn enable_attributes(&mut self, attributes: PciIoProtocolAttributes) -> Result {
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.attributes)(
                &mut self.0,
                PciIoProtocolAttributeOperation::ENABLE,
                attributes.bits(),
                ptr::null_mut(),
            )
            .to_result()
        }
    }

    /// Disables [`PciIoProtocolAttributes`] for this PCI controller.
    ///
    /// # Safety
    ///
    /// Disabling these attributes must be safe for the current system configuration.
    pub unsafe fn disable_attributes(&mut self, attributes: PciIoProtocolAttributes) -> Result {
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.attributes)(
                &mut self.0,
                PciIoProtocolAttributeOperation::DISABLE,
                attributes.bits(),
                ptr::null_mut(),
            )
            .to_result()
        }
    }

    /// Gets the attributes that this PCI controller supports setting on a BAR using
    /// [`Self::set_bar_attributes`].
    pub fn bar_attributes(&self, bar_index: u8) -> Result<PciIoProtocolAttributes> {
        let mut supports = 0;
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.get_bar_attributes)(&self.0, bar_index, &mut supports, ptr::null_mut())
                .to_result_with_val(|| PciIoProtocolAttributes::from_bits_retain(supports))
        }
    }

    /// Retrieves the current resource settings of the specified BAR on this PCI controller in the
    /// form of a set of ACPI resource descriptors.
    #[cfg(feature = "alloc")]
    pub fn bar_resources(
        &self,
        bar_index: u8,
    ) -> Result<alloc::vec::Vec<crate::proto::pci::configuration::QwordAddressSpaceDescriptor>>
    {
        use crate::proto::pci::configuration;
        let mut resources: *const core::ffi::c_void = ptr::null();
        // SAFETY: The memory is valid.
        unsafe {
            ((self.0.get_bar_attributes)(&self.0, bar_index, ptr::null_mut(), &mut resources))
                .to_result_with_val(|| configuration::parse(resources))
        }
    }

    /// Sets the attributes for a range of a BAR on a PCI controller.
    ///
    /// The provided offset and length are set to the actual base and length of the region whose
    /// attributes were changed.
    ///
    /// # Safety
    ///
    /// Modifying BAR attributes must be safe for the current hardware/system configuration.
    pub unsafe fn set_bar_attributes(
        &mut self,
        bar_index: u8,
        attributes: PciIoProtocolAttributes,
        offset: &mut u64,
        length: &mut u64,
    ) -> Result {
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.set_bar_attributes)(&mut self.0, attributes.bits(), bar_index, offset, length)
                .to_result()
        }
    }

    /// Returns the size, in bytes, of the Option ROM image.
    #[must_use]
    pub const fn rom_size(&self) -> u64 {
        self.0.rom_size
    }

    /// Returns a slice referencing the in-memory copy of the Option ROM image, or `None` if
    /// no Option ROM is present.
    #[must_use]
    pub fn rom_image(&self) -> Option<&[u8]> {
        if self.0.rom_image.is_null() || self.0.rom_size == 0 {
            None
        } else {
            let len = usize::try_from(self.0.rom_size).ok()?;
            // SAFETY: `rom_image` points to a buffer of `rom_size` bytes allocated by the PCI bus driver.
            Some(unsafe { core::slice::from_raw_parts(self.0.rom_image.cast(), len) })
        }
    }
}

/// Shared read/write plumbing behind [`PciIoConfigAccess`] and [`PciIoBarAccess`]. The two
/// only differ in how they address the underlying protocol call: configuration space access
/// takes a plain offset, while BAR access also needs a BAR index. Implementors provide the raw
/// FFI calls; the rest of the read/write/fill/fifo methods are derived from those once here.
trait PciIoRawAccess {
    /// The offset type used to address into this access kind.
    type Offset: Copy;

    fn raw_read<U: PciIoUnit>(
        &self,
        mode: PciIoMode,
        offset: Self::Offset,
        buffer: &mut [U],
    ) -> Status;

    fn raw_write<U: PciIoUnit>(
        &self,
        mode: PciIoMode,
        offset: Self::Offset,
        buffer: &[U],
    ) -> Status;

    /// `count` is not a buffer length here (see the trait-level doc comment): it tells firmware
    /// how many times to repeat writing `value` at the destination, so an oversized `count` is a
    /// firmware/device-level concern, not a host memory-safety one.
    fn raw_fill_write<U: PciIoUnit>(
        &self,
        mode: PciIoMode,
        offset: Self::Offset,
        count: usize,
        value: &U,
    ) -> Status;

    fn do_read_one<U: PciIoUnit>(&self, offset: Self::Offset) -> Result<U> {
        let mut result = U::default();
        self.raw_read(
            PciIoMode::Normal,
            offset,
            core::slice::from_mut(&mut result),
        )
        .to_result_with_val(|| result)
    }

    fn do_write_one<U: PciIoUnit>(&self, offset: Self::Offset, data: U) -> Result<()> {
        self.raw_write(PciIoMode::Normal, offset, core::slice::from_ref(&data))
            .to_result()
    }

    fn do_read<U: PciIoUnit>(&self, offset: Self::Offset, data: &mut [U]) -> Result<()> {
        self.raw_read(PciIoMode::Normal, offset, data).to_result()
    }

    fn do_write<U: PciIoUnit>(&self, offset: Self::Offset, data: &[U]) -> Result<()> {
        self.raw_write(PciIoMode::Normal, offset, data).to_result()
    }

    fn do_fill_write<U: PciIoUnit>(
        &self,
        offset: Self::Offset,
        count: usize,
        data: U,
    ) -> Result<()> {
        self.raw_fill_write(PciIoMode::Fill, offset, count, &data)
            .to_result()
    }

    fn do_fifo_read<U: PciIoUnit>(&self, offset: Self::Offset, data: &mut [U]) -> Result<()> {
        self.raw_read(PciIoMode::Fifo, offset, data).to_result()
    }

    fn do_fifo_write<U: PciIoUnit>(&self, offset: Self::Offset, data: &[U]) -> Result<()> {
        self.raw_write(PciIoMode::Fifo, offset, data).to_result()
    }
}

/// Struct for performing PCI Configuration Space I/O operations on a PCI controller.
#[derive(Debug)]
pub struct PciIoConfigAccess<'a> {
    proto: *mut PciIoProtocol,
    access: &'a mut PciIoProtocolConfigAccess,
}

impl PciIoRawAccess for PciIoConfigAccess<'_> {
    type Offset = u32;

    fn raw_read<U: PciIoUnit>(&self, mode: PciIoMode, offset: u32, buffer: &mut [U]) -> Status {
        let width = encode_pci_io_mode_and_unit::<U>(mode);
        // SAFETY: `self.proto` points to the `PciIoProtocol` embedded in a live `PciIo` (it is
        // only ever set from `&mut self.0` in `PciIo::pci`, a private field nothing outside this
        // module can override), so it is valid for this call. `buffer`/`buffer.len()` are a
        // matched pair straight from the Rust slice, and `width` is computed from the same `U`
        // as `buffer` just above, so it always matches.
        unsafe {
            (self.access.read)(
                self.proto,
                width,
                offset,
                buffer.len(),
                buffer.as_mut_ptr().cast(),
            )
        }
    }

    fn raw_write<U: PciIoUnit>(&self, mode: PciIoMode, offset: u32, buffer: &[U]) -> Status {
        let width = encode_pci_io_mode_and_unit::<U>(mode);
        // SAFETY: same reasoning as `raw_read` above, for the write direction.
        unsafe {
            (self.access.write)(
                self.proto,
                width,
                offset,
                buffer.len(),
                buffer.as_ptr().cast(),
            )
        }
    }

    fn raw_fill_write<U: PciIoUnit>(
        &self,
        mode: PciIoMode,
        offset: u32,
        count: usize,
        value: &U,
    ) -> Status {
        let width = encode_pci_io_mode_and_unit::<U>(mode);
        // SAFETY: `self.proto` validity as in `raw_read` above. `value` is a single valid `U` by
        // being an ordinary Rust reference; `width` matches `U` for the same reason as above;
        // `count` only tells firmware how many times to repeat the write at the destination, so
        // it does not describe `value`'s size.
        unsafe {
            (self.access.write)(
                self.proto,
                width,
                offset,
                count,
                ptr::from_ref(value).cast(),
            )
        }
    }
}

impl PciIoConfigAccess<'_> {
    /// Reads a single value of type `U` from the specified configuration space offset.
    pub fn read_one<U: PciIoUnit>(&self, offset: u32) -> Result<U> {
        self.do_read_one(offset)
    }

    /// Writes a single value of type `U` to the specified configuration space offset.
    pub fn write_one<U: PciIoUnit>(&self, offset: u32, data: U) -> Result<()> {
        self.do_write_one(offset, data)
    }

    /// Reads multiple values from the specified configuration space offset range.
    pub fn read<U: PciIoUnit>(&self, offset: u32, data: &mut [U]) -> Result<()> {
        self.do_read(offset, data)
    }

    /// Writes multiple values to the specified configuration space offset range.
    pub fn write<U: PciIoUnit>(&self, offset: u32, data: &[U]) -> Result<()> {
        self.do_write(offset, data)
    }

    /// Fills a configuration space offset range with the specified value.
    pub fn fill_write<U: PciIoUnit>(&self, offset: u32, count: usize, data: U) -> Result<()> {
        self.do_fill_write(offset, count, data)
    }

    /// Reads a sequence of values of type `U` from the specified configuration space offset by repeatedly accessing it.
    pub fn fifo_read<U: PciIoUnit>(&self, offset: u32, data: &mut [U]) -> Result<()> {
        self.do_fifo_read(offset, data)
    }

    /// Writes a sequence of values of type `U` to the specified configuration space offset repeatedly.
    pub fn fifo_write<U: PciIoUnit>(&self, offset: u32, data: &[U]) -> Result<()> {
        self.do_fifo_write(offset, data)
    }
}

/// Struct for performing BAR-based PCI Memory or I/O operations on a PCI controller.
#[derive(Debug)]
pub struct PciIoBarAccess<'a, S> {
    proto: *mut PciIoProtocol,
    access: &'a mut PciIoProtocolAccess,
    bar_index: u8,
    _address_space: S,
}

impl<S> PciIoRawAccess for PciIoBarAccess<'_, S> {
    type Offset = u64;

    fn raw_read<U: PciIoUnit>(&self, mode: PciIoMode, offset: u64, buffer: &mut [U]) -> Status {
        let width = encode_pci_io_mode_and_unit::<U>(mode);
        // SAFETY: `self.proto` points to the `PciIoProtocol` embedded in a live `PciIo` (it is
        // only ever set from `&mut self.0` in `PciIo::memory`/`PciIo::io`, private fields nothing
        // outside this module can override), so it is valid for this call. `buffer`/`buffer.len()`
        // are a matched pair straight from the Rust slice, and `width` is computed from the same
        // `U` as `buffer` just above, so it always matches.
        unsafe {
            (self.access.read)(
                self.proto,
                width,
                self.bar_index,
                offset,
                buffer.len(),
                buffer.as_mut_ptr().cast(),
            )
        }
    }

    fn raw_write<U: PciIoUnit>(&self, mode: PciIoMode, offset: u64, buffer: &[U]) -> Status {
        let width = encode_pci_io_mode_and_unit::<U>(mode);
        // SAFETY: same reasoning as `raw_read` above, for the write direction.
        unsafe {
            (self.access.write)(
                self.proto,
                width,
                self.bar_index,
                offset,
                buffer.len(),
                buffer.as_ptr().cast(),
            )
        }
    }

    fn raw_fill_write<U: PciIoUnit>(
        &self,
        mode: PciIoMode,
        offset: u64,
        count: usize,
        value: &U,
    ) -> Status {
        let width = encode_pci_io_mode_and_unit::<U>(mode);
        // SAFETY: `self.proto` validity as in `raw_read` above. `value` is a single valid `U` by
        // being an ordinary Rust reference; `width` matches `U` for the same reason as above;
        // `count` only tells firmware how many times to repeat the write at the destination, so
        // it does not describe `value`'s size.
        unsafe {
            (self.access.write)(
                self.proto,
                width,
                self.bar_index,
                offset,
                count,
                ptr::from_ref(value).cast(),
            )
        }
    }
}

impl<S> PciIoBarAccess<'_, S> {
    /// Reads a single value of type `U` from the specified BAR offset.
    pub fn read_one<U: PciIoUnit>(&self, offset: u64) -> Result<U> {
        self.do_read_one(offset)
    }

    /// Writes a single value of type `U` to the specified BAR offset.
    pub fn write_one<U: PciIoUnit>(&self, offset: u64, data: U) -> Result<()> {
        self.do_write_one(offset, data)
    }

    /// Reads multiple values from the specified BAR offset range.
    pub fn read<U: PciIoUnit>(&self, offset: u64, data: &mut [U]) -> Result<()> {
        self.do_read(offset, data)
    }

    /// Writes multiple values to the specified BAR offset range.
    pub fn write<U: PciIoUnit>(&self, offset: u64, data: &[U]) -> Result<()> {
        self.do_write(offset, data)
    }

    /// Fills a BAR offset range with the specified value.
    pub fn fill_write<U: PciIoUnit>(&self, offset: u64, count: usize, data: U) -> Result<()> {
        self.do_fill_write(offset, count, data)
    }

    /// Reads a sequence of values of type `U` from the specified BAR offset by repeatedly accessing it.
    pub fn fifo_read<U: PciIoUnit>(&self, offset: u64, data: &mut [U]) -> Result<()> {
        self.do_fifo_read(offset, data)
    }

    /// Writes a sequence of values of type `U` to the specified BAR offset repeatedly.
    pub fn fifo_write<U: PciIoUnit>(&self, offset: u64, data: &[U]) -> Result<()> {
        self.do_fifo_write(offset, data)
    }
}
