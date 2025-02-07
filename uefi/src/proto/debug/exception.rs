// SPDX-License-Identifier: MIT OR Apache-2.0

/// Represents supported CPU exceptions.
#[repr(C)]
#[derive(Debug)]
pub struct ExceptionType(isize);

impl ExceptionType {
    /// Undefined Exception
    pub const EXCEPT_EBC_UNDEFINED: Self = Self(0);
    /// Divide-by-zero Error
    pub const EXCEPT_EBC_DIVIDE_ERROR: Self = Self(1);
    /// Debug Exception
    pub const EXCEPT_EBC_DEBUG: Self = Self(2);
    /// Breakpoint
    pub const EXCEPT_EBC_BREAKPOINT: Self = Self(3);
    /// Overflow
    pub const EXCEPT_EBC_OVERFLOW: Self = Self(4);
    /// Invalid Opcode
    pub const EXCEPT_EBC_INVALID_OPCODE: Self = Self(5);
    /// Stack-Segment Fault
    pub const EXCEPT_EBC_STACK_FAULT: Self = Self(6);
    /// Alignment Check
    pub const EXCEPT_EBC_ALIGNMENT_CHECK: Self = Self(7);
    /// Instruction Encoding Exception
    pub const EXCEPT_EBC_INSTRUCTION_ENCODING: Self = Self(8);
    /// Bad Breakpoint Exception
    pub const EXCEPT_EBC_BAD_BREAK: Self = Self(9);
    /// Single Step Exception
    pub const EXCEPT_EBC_SINGLE_STEP: Self = Self(10);
}

#[cfg(target_arch = "x86")]
impl ExceptionType {
    /// Divide-by-zero Error
    pub const EXCEPT_IA32_DIVIDE_ERROR: Self = Self(0);
    /// Debug Exception
    pub const EXCEPT_IA32_DEBUG: Self = Self(1);
    /// Non-maskable Interrupt
    pub const EXCEPT_IA32_NMI: Self = Self(2);
    /// Breakpoint
    pub const EXCEPT_IA32_BREAKPOINT: Self = Self(3);
    /// Overflow
    pub const EXCEPT_IA32_OVERFLOW: Self = Self(4);
    /// Bound Range Exceeded
    pub const EXCEPT_IA32_BOUND: Self = Self(5);
    /// Invalid Opcode
    pub const EXCEPT_IA32_INVALID_OPCODE: Self = Self(6);
    /// Double Fault
    pub const EXCEPT_IA32_DOUBLE_FAULT: Self = Self(8);
    /// Invalid TSS
    pub const EXCEPT_IA32_INVALID_TSS: Self = Self(10);
    /// Segment Not Present
    pub const EXCEPT_IA32_SEG_NOT_PRESENT: Self = Self(11);
    /// Stack-Segment Fault
    pub const EXCEPT_IA32_STACK_FAULT: Self = Self(12);
    /// General Protection Fault
    pub const EXCEPT_IA32_GP_FAULT: Self = Self(13);
    /// Page Fault
    pub const EXCEPT_IA32_PAGE_FAULT: Self = Self(14);
    /// x87 Floating-Point Exception
    pub const EXCEPT_IA32_FP_ERROR: Self = Self(16);
    /// Alignment Check
    pub const EXCEPT_IA32_ALIGNMENT_CHECK: Self = Self(17);
    /// Machine Check
    pub const EXCEPT_IA32_MACHINE_CHECK: Self = Self(18);
    /// SIMD Floating-Point Exception
    pub const EXCEPT_IA32_SIMD: Self = Self(19);
}

#[cfg(target_arch = "x86_64")]
impl ExceptionType {
    /// Divide-by-zero Error
    pub const EXCEPT_X64_DIVIDE_ERROR: Self = Self(0);
    /// Debug Exception
    pub const EXCEPT_X64_DEBUG: Self = Self(1);
    /// Non-maskable Interrupt
    pub const EXCEPT_X64_NMI: Self = Self(2);
    /// Breakpoint
    pub const EXCEPT_X64_BREAKPOINT: Self = Self(3);
    /// Overflow
    pub const EXCEPT_X64_OVERFLOW: Self = Self(4);
    /// Bound Range Exceeded
    pub const EXCEPT_X64_BOUND: Self = Self(5);
    /// Invalid Opcode
    pub const EXCEPT_X64_INVALID_OPCODE: Self = Self(6);
    /// Double Fault
    pub const EXCEPT_X64_DOUBLE_FAULT: Self = Self(8);
    /// Invalid TSS
    pub const EXCEPT_X64_INVALID_TSS: Self = Self(10);
    /// Segment Not Present
    pub const EXCEPT_X64_SEG_NOT_PRESENT: Self = Self(11);
    /// Stack-Segment Fault
    pub const EXCEPT_X64_STACK_FAULT: Self = Self(12);
    /// General Protection Fault
    pub const EXCEPT_X64_GP_FAULT: Self = Self(13);
    /// Page Fault
    pub const EXCEPT_X64_PAGE_FAULT: Self = Self(14);
    /// x87 Floating-Point Exception
    pub const EXCEPT_X64_FP_ERROR: Self = Self(16);
    /// Alignment Check
    pub const EXCEPT_X64_ALIGNMENT_CHECK: Self = Self(17);
    /// Machine Check
    pub const EXCEPT_X64_MACHINE_CHECK: Self = Self(18);
    /// SIMD Floating-Point Exception
    pub const EXCEPT_X64_SIMD: Self = Self(19);
}

#[cfg(target_arch = "arm")]
impl ExceptionType {
    /// Processor reset
    pub const EXCEPT_ARM_RESET: Self = Self(0);
    /// Undefined instruction
    pub const EXCEPT_ARM_UNDEFINED_INSTRUCTION: Self = Self(1);
    /// Software Interrupt
    pub const EXCEPT_ARM_SOFTWARE_INTERRUPT: Self = Self(2);
    /// Prefetch aborted
    pub const EXCEPT_ARM_PREFETCH_ABORT: Self = Self(3);
    /// Data access memory abort
    pub const EXCEPT_ARM_DATA_ABORT: Self = Self(4);
    /// Reserved
    pub const EXCEPT_ARM_RESERVED: Self = Self(5);
    /// Normal interrupt
    pub const EXCEPT_ARM_IRQ: Self = Self(6);
    /// Fast interrupt
    pub const EXCEPT_ARM_FIQ: Self = Self(7);
    /// In the UEFI spec for "convenience", unsure if we'll need it. Set to `EXCEPT_ARM_FIQ`
    pub const MAX_ARM_EXCEPTION: Self = Self::EXCEPT_ARM_FIQ;
}

#[cfg(target_arch = "aarch64")]
impl ExceptionType {
    /// Synchronous exception, such as attempting to execute an invalid instruction
    pub const EXCEPT_AARCH64_SYNCHRONOUS_EXCEPTIONS: Self = Self(0);
    /// Normal interrupt
    pub const EXCEPT_AARCH64_IRQ: Self = Self(1);
    /// Fast interrupt
    pub const EXCEPT_AARCH64_FIQ: Self = Self(2);
    /// System Error
    pub const EXCEPT_AARCH64_SERROR: Self = Self(3);
    /// In the UEFI spec for "convenience", unsure if we'll need it. Set to `EXCEPT_AARCH64_SERROR`
    pub const MAX_AARCH64_EXCEPTION: Self = Self::EXCEPT_AARCH64_SERROR;
}

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
impl ExceptionType {
    /// Instruction misaligned
    pub const EXCEPT_RISCV_INST_MISALIGNED: Self = Self(0);
    /// Instruction access fault
    pub const EXCEPT_RISCV_INST_ACCESS_FAULT: Self = Self(1);
    /// Illegal instruction
    pub const EXCEPT_RISCV_ILLEGAL_INST: Self = Self(2);
    /// Breakpoint
    pub const EXCEPT_RISCV_BREAKPOINT: Self = Self(3);
    /// Load address misaligned
    pub const EXCEPT_RISCV_LOAD_ADDRESS_MISALIGNED: Self = Self(4);
    /// Load accept fault
    pub const EXCEPT_RISCV_LOAD_ACCESS_FAULT: Self = Self(5);
    /// Store AMO address misaligned
    pub const EXCEPT_RISCV_STORE_AMO_ADDRESS_MISALIGNED: Self = Self(6);
    /// Store AMO access fault
    pub const EXCEPT_RISCV_STORE_AMO_ACCESS_FAULT: Self = Self(7);
    /// ECALL from User mode
    pub const EXCEPT_RISCV_ENV_CALL_FROM_UMODE: Self = Self(8);
    /// ECALL from Supervisor mode
    pub const EXCEPT_RISCV_ENV_CALL_FROM_SMODE: Self = Self(9);
    /// ECALL from Machine mode
    pub const EXCEPT_RISCV_ENV_CALL_FROM_MMODE: Self = Self(11);
    /// Instruction page fault
    pub const EXCEPT_RISCV_INST_PAGE_FAULT: Self = Self(12);
    /// Load page fault
    pub const EXCEPT_RISCV_LOAD_PAGE_FAULT: Self = Self(13);
    /// Store AMO page fault
    pub const EXCEPT_RISCV_STORE_AMO_PAGE_FAULT: Self = Self(15);
    // RISC-V interrupt types
    /// Supervisor software interrupt
    pub const EXCEPT_RISCV_SUPERVISOR_SOFTWARE_INT: Self = Self(1);
    /// Machine software interrupt
    pub const EXCEPT_RISCV_MACHINE_SOFTWARE_INT: Self = Self(3);
    /// Supervisor timer interrupt
    pub const EXCEPT_RISCV_SUPERVISOR_TIMER_INT: Self = Self(5);
    /// Machine timer interrupt
    pub const EXCEPT_RISCV_MACHINE_TIMER_INT: Self = Self(7);
    /// Supervisor external interrupt
    pub const EXCEPT_RISCV_SUPERVISOR_EXTERNAL_INT: Self = Self(9);
    /// Machine external interrupt
    pub const EXCEPT_RISCV_MACHINE_EXTERNAL_INT: Self = Self(11);
}
