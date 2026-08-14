// SPDX-License-Identifier: MIT OR Apache-2.0

/// Represents supported CPU exceptions.
#[repr(C)]
#[derive(Debug)]
pub struct ExceptionType(isize);

impl ExceptionType {
    /// Indicates an undefined EBC exception.
    pub const EXCEPT_EBC_UNDEFINED: Self = Self(0);
    /// Indicates an EBC divide-by-zero error.
    pub const EXCEPT_EBC_DIVIDE_ERROR: Self = Self(1);
    /// Indicates an EBC debug exception.
    pub const EXCEPT_EBC_DEBUG: Self = Self(2);
    /// Indicates an EBC breakpoint.
    pub const EXCEPT_EBC_BREAKPOINT: Self = Self(3);
    /// Indicates an EBC overflow.
    pub const EXCEPT_EBC_OVERFLOW: Self = Self(4);
    /// Indicates an invalid EBC opcode.
    pub const EXCEPT_EBC_INVALID_OPCODE: Self = Self(5);
    /// Indicates an EBC stack fault.
    pub const EXCEPT_EBC_STACK_FAULT: Self = Self(6);
    /// Indicates an EBC alignment check.
    pub const EXCEPT_EBC_ALIGNMENT_CHECK: Self = Self(7);
    /// Indicates an EBC instruction-encoding exception.
    pub const EXCEPT_EBC_INSTRUCTION_ENCODING: Self = Self(8);
    /// Indicates an invalid EBC breakpoint.
    pub const EXCEPT_EBC_BAD_BREAK: Self = Self(9);
    /// Indicates an EBC single-step exception.
    pub const EXCEPT_EBC_SINGLE_STEP: Self = Self(10);
}

#[cfg(target_arch = "x86")]
impl ExceptionType {
    /// Indicates a divide-by-zero error.
    pub const EXCEPT_IA32_DIVIDE_ERROR: Self = Self(0);
    /// Indicates a debug exception.
    pub const EXCEPT_IA32_DEBUG: Self = Self(1);
    /// Indicates a non-maskable interrupt.
    pub const EXCEPT_IA32_NMI: Self = Self(2);
    /// Indicates a breakpoint.
    pub const EXCEPT_IA32_BREAKPOINT: Self = Self(3);
    /// Indicates an overflow.
    pub const EXCEPT_IA32_OVERFLOW: Self = Self(4);
    /// Indicates that a bound range was exceeded.
    pub const EXCEPT_IA32_BOUND: Self = Self(5);
    /// Indicates an invalid opcode.
    pub const EXCEPT_IA32_INVALID_OPCODE: Self = Self(6);
    /// Indicates a double fault.
    pub const EXCEPT_IA32_DOUBLE_FAULT: Self = Self(8);
    /// Indicates an invalid task-state segment.
    pub const EXCEPT_IA32_INVALID_TSS: Self = Self(10);
    /// Indicates a missing segment.
    pub const EXCEPT_IA32_SEG_NOT_PRESENT: Self = Self(11);
    /// Indicates a stack-segment fault.
    pub const EXCEPT_IA32_STACK_FAULT: Self = Self(12);
    /// Indicates a general-protection fault.
    pub const EXCEPT_IA32_GP_FAULT: Self = Self(13);
    /// Indicates a page fault.
    pub const EXCEPT_IA32_PAGE_FAULT: Self = Self(14);
    /// Indicates an x87 floating-point exception.
    pub const EXCEPT_IA32_FP_ERROR: Self = Self(16);
    /// Indicates an alignment check.
    pub const EXCEPT_IA32_ALIGNMENT_CHECK: Self = Self(17);
    /// Indicates a machine check.
    pub const EXCEPT_IA32_MACHINE_CHECK: Self = Self(18);
    /// Indicates a SIMD floating-point exception.
    pub const EXCEPT_IA32_SIMD: Self = Self(19);
}

#[cfg(target_arch = "x86_64")]
impl ExceptionType {
    /// Indicates a divide-by-zero error.
    pub const EXCEPT_X64_DIVIDE_ERROR: Self = Self(0);
    /// Indicates a debug exception.
    pub const EXCEPT_X64_DEBUG: Self = Self(1);
    /// Indicates a non-maskable interrupt.
    pub const EXCEPT_X64_NMI: Self = Self(2);
    /// Indicates a breakpoint.
    pub const EXCEPT_X64_BREAKPOINT: Self = Self(3);
    /// Indicates an overflow.
    pub const EXCEPT_X64_OVERFLOW: Self = Self(4);
    /// Indicates that a bound range was exceeded.
    pub const EXCEPT_X64_BOUND: Self = Self(5);
    /// Indicates an invalid opcode.
    pub const EXCEPT_X64_INVALID_OPCODE: Self = Self(6);
    /// Indicates a double fault.
    pub const EXCEPT_X64_DOUBLE_FAULT: Self = Self(8);
    /// Indicates an invalid task-state segment.
    pub const EXCEPT_X64_INVALID_TSS: Self = Self(10);
    /// Indicates a missing segment.
    pub const EXCEPT_X64_SEG_NOT_PRESENT: Self = Self(11);
    /// Indicates a stack-segment fault.
    pub const EXCEPT_X64_STACK_FAULT: Self = Self(12);
    /// Indicates a general-protection fault.
    pub const EXCEPT_X64_GP_FAULT: Self = Self(13);
    /// Indicates a page fault.
    pub const EXCEPT_X64_PAGE_FAULT: Self = Self(14);
    /// Indicates an x87 floating-point exception.
    pub const EXCEPT_X64_FP_ERROR: Self = Self(16);
    /// Indicates an alignment check.
    pub const EXCEPT_X64_ALIGNMENT_CHECK: Self = Self(17);
    /// Indicates a machine check.
    pub const EXCEPT_X64_MACHINE_CHECK: Self = Self(18);
    /// Indicates a SIMD floating-point exception.
    pub const EXCEPT_X64_SIMD: Self = Self(19);
}

#[cfg(target_arch = "arm")]
impl ExceptionType {
    /// Indicates a processor reset.
    pub const EXCEPT_ARM_RESET: Self = Self(0);
    /// Indicates an undefined instruction.
    pub const EXCEPT_ARM_UNDEFINED_INSTRUCTION: Self = Self(1);
    /// Indicates a software interrupt.
    pub const EXCEPT_ARM_SOFTWARE_INTERRUPT: Self = Self(2);
    /// Indicates an aborted prefetch.
    pub const EXCEPT_ARM_PREFETCH_ABORT: Self = Self(3);
    /// Indicates a data-access memory abort.
    pub const EXCEPT_ARM_DATA_ABORT: Self = Self(4);
    /// Represents the reserved exception value.
    pub const EXCEPT_ARM_RESERVED: Self = Self(5);
    /// Indicates a normal interrupt.
    pub const EXCEPT_ARM_IRQ: Self = Self(6);
    /// Indicates a fast interrupt.
    pub const EXCEPT_ARM_FIQ: Self = Self(7);
    /// Provides the maximum ARM exception value defined by UEFI.
    pub const MAX_ARM_EXCEPTION: Self = Self::EXCEPT_ARM_FIQ;
}

#[cfg(target_arch = "aarch64")]
impl ExceptionType {
    /// Indicates a synchronous exception, such as an invalid instruction.
    pub const EXCEPT_AARCH64_SYNCHRONOUS_EXCEPTIONS: Self = Self(0);
    /// Indicates a normal interrupt.
    pub const EXCEPT_AARCH64_IRQ: Self = Self(1);
    /// Indicates a fast interrupt.
    pub const EXCEPT_AARCH64_FIQ: Self = Self(2);
    /// Indicates a system error.
    pub const EXCEPT_AARCH64_SERROR: Self = Self(3);
    /// Provides the maximum AArch64 exception value defined by UEFI.
    pub const MAX_AARCH64_EXCEPTION: Self = Self::EXCEPT_AARCH64_SERROR;
}

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
impl ExceptionType {
    /// Indicates a misaligned instruction address.
    pub const EXCEPT_RISCV_INST_MISALIGNED: Self = Self(0);
    /// Indicates an instruction-access fault.
    pub const EXCEPT_RISCV_INST_ACCESS_FAULT: Self = Self(1);
    /// Indicates an illegal instruction.
    pub const EXCEPT_RISCV_ILLEGAL_INST: Self = Self(2);
    /// Indicates a breakpoint.
    pub const EXCEPT_RISCV_BREAKPOINT: Self = Self(3);
    /// Indicates a misaligned load address.
    pub const EXCEPT_RISCV_LOAD_ADDRESS_MISALIGNED: Self = Self(4);
    /// Indicates a load-access fault.
    pub const EXCEPT_RISCV_LOAD_ACCESS_FAULT: Self = Self(5);
    /// Indicates a misaligned store or AMO address.
    pub const EXCEPT_RISCV_STORE_AMO_ADDRESS_MISALIGNED: Self = Self(6);
    /// Indicates a store or AMO access fault.
    pub const EXCEPT_RISCV_STORE_AMO_ACCESS_FAULT: Self = Self(7);
    /// Indicates an environment call from user mode.
    pub const EXCEPT_RISCV_ENV_CALL_FROM_UMODE: Self = Self(8);
    /// Indicates an environment call from supervisor mode.
    pub const EXCEPT_RISCV_ENV_CALL_FROM_SMODE: Self = Self(9);
    /// Indicates an environment call from machine mode.
    pub const EXCEPT_RISCV_ENV_CALL_FROM_MMODE: Self = Self(11);
    /// Indicates an instruction page fault.
    pub const EXCEPT_RISCV_INST_PAGE_FAULT: Self = Self(12);
    /// Indicates a load page fault.
    pub const EXCEPT_RISCV_LOAD_PAGE_FAULT: Self = Self(13);
    /// Indicates a store or AMO page fault.
    pub const EXCEPT_RISCV_STORE_AMO_PAGE_FAULT: Self = Self(15);
    // RISC-V interrupt types
    /// Indicates a supervisor software interrupt.
    pub const EXCEPT_RISCV_SUPERVISOR_SOFTWARE_INT: Self = Self(1);
    /// Indicates a machine software interrupt.
    pub const EXCEPT_RISCV_MACHINE_SOFTWARE_INT: Self = Self(3);
    /// Indicates a supervisor timer interrupt.
    pub const EXCEPT_RISCV_SUPERVISOR_TIMER_INT: Self = Self(5);
    /// Indicates a machine timer interrupt.
    pub const EXCEPT_RISCV_MACHINE_TIMER_INT: Self = Self(7);
    /// Indicates a supervisor external interrupt.
    pub const EXCEPT_RISCV_SUPERVISOR_EXTERNAL_INT: Self = Self(9);
    /// Indicates a machine external interrupt.
    pub const EXCEPT_RISCV_MACHINE_EXTERNAL_INT: Self = Self(11);
}
