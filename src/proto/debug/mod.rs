//! Provides support for the UEFI debugging protocol.
//!
//! This protocol is designed to allow debuggers to query the state of the firmware,
//! as well as set up callbacks for various events.
//!
//! It also defines a Debugport protocol for debugging over serial devices.
//!
//! An example UEFI debugger is Intel's [UDK Debugger Tool][udk].
//!
//! [udk]: https://firmware.intel.com/develop/intel-uefi-tools-and-utilities/intel-uefi-development-kit-debugger-tool

use core::ffi::c_void;

use crate::proto::Protocol;
use crate::{unsafe_guid, Result, Status};

// re-export for ease of use
pub use self::context::SystemContext;
pub use self::exception::ExceptionType;

mod context;
mod exception;

/// The debugging support protocol allows debuggers to connect to a UEFI machine.
/// It is expected that there will typically be two instances of the EFI Debug Support protocol in the system.
/// One associated with the native processor instruction set (IA-32, x64, ARM, RISC-V, or Itanium processor
/// family), and one for the EFI virtual machine that implements EFI byte code (EBC).
/// While multiple instances of the EFI Debug Support protocol are expected, there must never be more than
/// one for any given instruction set.
///
/// NOTE: OVMF only implements this protocol interface for the virtual EBC processor
#[repr(C)]
#[unsafe_guid("2755590c-6f3c-42fa-9ea4-a3ba543cda25")]
#[derive(Protocol)]
pub struct DebugSupport {
    isa: ProcessorArch,
    get_maximum_processor_index:
        extern "efiapi" fn(this: &mut DebugSupport, max_processor_index: &mut usize) -> Status,
    register_periodic_callback: extern "efiapi" fn(
        this: &mut DebugSupport,
        processor_index: usize,
        periodic_callback: Option<extern "efiapi" fn(SystemContext)>,
    ) -> Status,
    register_exception_callback: extern "efiapi" fn(
        this: &mut DebugSupport,
        processor_index: usize,
        exception_callback: Option<extern "efiapi" fn(ExceptionType, SystemContext)>,
        exception_type: ExceptionType,
    ) -> Status,
    invalidate_instruction_cache: extern "efiapi" fn(
        this: &mut DebugSupport,
        processor_index: usize,
        start: *mut c_void,
        length: u64,
    ) -> Status,
}

impl DebugSupport {
    /// Returns the processor architecture of the running CPU.
    pub fn arch(&self) -> ProcessorArch {
        self.isa
    }

    /// Returns the maximum value that may be used for the processor_index parameter in
    /// `register_periodic_callback()` and `register_exception_callback()`.
    ///
    /// Note: Applications built with EDK2 (such as OVMF) always return `0` as of 2021-09-15
    pub fn get_maximum_processor_index(&mut self) -> usize {
        // initially set to a canary value for testing purposes
        let mut max_processor_index: usize = usize::MAX;

        // per the UEFI spec, this method can ONLY return EFI_SUCCESS
        match (self.get_maximum_processor_index)(self, &mut max_processor_index) {
            Status::SUCCESS => max_processor_index,
            _ => unreachable!(), // panic in the case of incorrect implementations
        }
    }

    /// Registers a function to be called back periodically in interrupt context.
    /// Pass `None` for `callback` to deregister the currently registered function for
    /// `processor_index`.
    ///
    /// Note: Applications built with EDK2 (such as OVMF) ignore the `processor_index` parameter
    ///
    /// # Safety
    /// It is the responsibility of the caller to ensure that all parameters are correct. There are no
    /// provisions in this function or in the specification for parameter checking.
    pub unsafe fn register_periodic_callback(
        &mut self,
        processor_index: usize,
        callback: Option<extern "efiapi" fn(SystemContext)>,
    ) -> Result {
        (self.register_periodic_callback)(self, processor_index, callback).into()
    }

    /// Registers a function to be called when a given processor exception occurs.
    /// Pass `None` for `callback` to deregister the currently registered function for a
    /// given processor exception.
    ///
    /// Note: Applications built with EDK2 (such as OVMF) ignore the `processor_index` parameter
    ///
    /// # Safety
    /// It is the responsibility of the caller to ensure that all parameters are correct. There are no
    /// provisions in this function or in the specification for parameter checking.
    pub unsafe fn register_exception_callback(
        &mut self,
        processor_index: usize,
        callback: Option<extern "efiapi" fn(ExceptionType, SystemContext)>,
        exception_type: ExceptionType,
    ) -> Result {
        (self.register_exception_callback)(self, processor_index, callback, exception_type).into()
    }

    /// Invalidates processor instruction cache for a memory range.
    ///
    /// Note: Applications built with EDK2 (such as OVMF) ignore the `processor_index` parameter
    ///
    /// # Safety
    /// It is the responsibility of the caller to ensure that all parameters are correct. There are no
    /// provisions in this function or in the specification for parameter checking.
    pub unsafe fn invalidate_instruction_cache(
        &mut self,
        processor_index: usize,
        start: *mut c_void,
        length: u64,
    ) -> Result {
        // per the UEFI spec, this method can ONLY return EFI_SUCCESS
        (self.invalidate_instruction_cache)(self, processor_index, start, length).into()
    }
}

newtype_enum! {
/// The instruction set architecture of the running processor.
///
/// UEFI can be and has been ported to new CPU architectures in the past,
/// therefore modeling this C enum as a Rust enum (where the compiler must know
/// about every variant in existence) would _not_ be safe.
pub enum ProcessorArch: u32 => {
    /// 32-bit x86 PC
    X86_32      = 0x014C,
    /// 64-bit x86 PC
    X86_64      = 0x8664,
    /// Intel Itanium
    ITANIUM     = 0x200,
    /// UEFI Interpreter bytecode
    EBC         = 0x0EBC,
    /// ARM Thumb / Mixed
    ARM         = 0x01C2,
    /// ARM 64-bit
    AARCH_64    = 0xAA64,
    /// RISC-V 32-bit
    RISCV_32    = 0x5032,
    /// RISC-V 64-bit
    RISCV_64    = 0x5064,
    /// RISC-V 128-bit
    RISCV_128   = 0x5128,
}}
