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

/// The debugging support protocol allows debuggers to connect to a UEFI machine.
#[repr(C)]
pub struct DebugSupport {
    isa: ProcessorArch,
}

impl DebugSupport {
    /// Returns the processor architecture of the running CPU.
    pub fn arch(&self) -> ProcessorArch {
        self.isa
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

impl_proto! {
    protocol DebugSupport {
        GUID = 0x2755590C, 0x6F3C, 0x42FA, [0x9E, 0xA4, 0xA3, 0xBA, 0x54, 0x3C, 0xDA, 0x25];
    }
}
