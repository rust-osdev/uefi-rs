/// Represents supported CPU exceptions.
#[repr(C)]
pub struct ExceptionType(isize);

impl ExceptionType {
    /// Undefined Exception
    pub const EXCEPT_EBC_UNDEFINED: ExceptionType = ExceptionType(0);
    /// Divide-by-zero Error
    pub const EXCEPT_EBC_DIVIDE_ERROR: ExceptionType = ExceptionType(1);
    /// Debug Exception
    pub const EXCEPT_EBC_DEBUG: ExceptionType = ExceptionType(2);
    /// Breakpoint
    pub const EXCEPT_EBC_BREAKPOINT: ExceptionType = ExceptionType(3);
    /// Overflow
    pub const EXCEPT_EBC_OVERFLOW: ExceptionType = ExceptionType(4);
    /// Invalid Opcode
    pub const EXCEPT_EBC_INVALID_OPCODE: ExceptionType = ExceptionType(5);
    /// Stack-Segment Fault
    pub const EXCEPT_EBC_STACK_FAULT: ExceptionType = ExceptionType(6);
    /// Alignment Check
    pub const EXCEPT_EBC_ALIGNMENT_CHECK: ExceptionType = ExceptionType(7);
    /// Instruction Encoding Exception
    pub const EXCEPT_EBC_INSTRUCTION_ENCODING: ExceptionType = ExceptionType(8);
    /// Bad Breakpoint Exception
    pub const EXCEPT_EBC_BAD_BREAK: ExceptionType = ExceptionType(9);
    /// Single Step Exception
    pub const EXCEPT_EBC_SINGLE_STEP: ExceptionType = ExceptionType(10);
}

#[cfg(target_arch = "x86")]
impl ExceptionType {
    /// Divide-by-zero Error
    pub const EXCEPT_IA32_DIVIDE_ERROR: ExceptionType = ExceptionType(0);
    /// Debug Exception
    pub const EXCEPT_IA32_DEBUG: ExceptionType = ExceptionType(1);
    /// Non-maskable Interrupt
    pub const EXCEPT_IA32_NMI: ExceptionType = ExceptionType(2);
    /// Breakpoint
    pub const EXCEPT_IA32_BREAKPOINT: ExceptionType = ExceptionType(3);
    /// Overflow
    pub const EXCEPT_IA32_OVERFLOW: ExceptionType = ExceptionType(4);
    /// Bound Range Exceeded
    pub const EXCEPT_IA32_BOUND: ExceptionType = ExceptionType(5);
    /// Invalid Opcode
    pub const EXCEPT_IA32_INVALID_OPCODE: ExceptionType = ExceptionType(6);
    /// Double Fault
    pub const EXCEPT_IA32_DOUBLE_FAULT: ExceptionType = ExceptionType(8);
    /// Invalid TSS
    pub const EXCEPT_IA32_INVALID_TSS: ExceptionType = ExceptionType(10);
    /// Segment Not Present
    pub const EXCEPT_IA32_SEG_NOT_PRESENT: ExceptionType = ExceptionType(11);
    /// Stack-Segment Fault
    pub const EXCEPT_IA32_STACK_FAULT: ExceptionType = ExceptionType(12);
    /// General Protection Fault
    pub const EXCEPT_IA32_GP_FAULT: ExceptionType = ExceptionType(13);
    /// Page Fault
    pub const EXCEPT_IA32_PAGE_FAULT: ExceptionType = ExceptionType(14);
    /// x87 Floating-Point Exception
    pub const EXCEPT_IA32_FP_ERROR: ExceptionType = ExceptionType(16);
    /// Alignment Check
    pub const EXCEPT_IA32_ALIGNMENT_CHECK: ExceptionType = ExceptionType(17);
    /// Machine Check
    pub const EXCEPT_IA32_MACHINE_CHECK: ExceptionType = ExceptionType(18);
    /// SIMD Floating-Point Exception
    pub const EXCEPT_IA32_SIMD: ExceptionType = ExceptionType(19);
}

#[cfg(target_arch = "x86_64")]
impl ExceptionType {
    /// Divide-by-zero Error
    pub const EXCEPT_X64_DIVIDE_ERROR: ExceptionType = ExceptionType(0);
    /// Debug Exception
    pub const EXCEPT_X64_DEBUG: ExceptionType = ExceptionType(1);
    /// Non-maskable Interrupt
    pub const EXCEPT_X64_NMI: ExceptionType = ExceptionType(2);
    /// Breakpoint
    pub const EXCEPT_X64_BREAKPOINT: ExceptionType = ExceptionType(3);
    /// Overflow
    pub const EXCEPT_X64_OVERFLOW: ExceptionType = ExceptionType(4);
    /// Bound Range Exceeded
    pub const EXCEPT_X64_BOUND: ExceptionType = ExceptionType(5);
    /// Invalid Opcode
    pub const EXCEPT_X64_INVALID_OPCODE: ExceptionType = ExceptionType(6);
    /// Double Fault
    pub const EXCEPT_X64_DOUBLE_FAULT: ExceptionType = ExceptionType(8);
    /// Invalid TSS
    pub const EXCEPT_X64_INVALID_TSS: ExceptionType = ExceptionType(10);
    /// Segment Not Present
    pub const EXCEPT_X64_SEG_NOT_PRESENT: ExceptionType = ExceptionType(11);
    /// Stack-Segment Fault
    pub const EXCEPT_X64_STACK_FAULT: ExceptionType = ExceptionType(12);
    /// General Protection Fault
    pub const EXCEPT_X64_GP_FAULT: ExceptionType = ExceptionType(13);
    /// Page Fault
    pub const EXCEPT_X64_PAGE_FAULT: ExceptionType = ExceptionType(14);
    /// x87 Floating-Point Exception
    pub const EXCEPT_X64_FP_ERROR: ExceptionType = ExceptionType(16);
    /// Alignment Check
    pub const EXCEPT_X64_ALIGNMENT_CHECK: ExceptionType = ExceptionType(17);
    /// Machine Check
    pub const EXCEPT_X64_MACHINE_CHECK: ExceptionType = ExceptionType(18);
    /// SIMD Floating-Point Exception
    pub const EXCEPT_X64_SIMD: ExceptionType = ExceptionType(19);
}

#[cfg(target_arch = "arm")]
impl ExceptionType {
    /// Processor reset
    pub const EXCEPT_ARM_RESET: ExceptionType = ExceptionType(0);
    /// Undefined instruction
    pub const EXCEPT_ARM_UNDEFINED_INSTRUCTION: ExceptionType = ExceptionType(1);
    /// Software Interrupt
    pub const EXCEPT_ARM_SOFTWARE_INTERRUPT: ExceptionType = ExceptionType(2);
    /// Prefetch aborted
    pub const EXCEPT_ARM_PREFETCH_ABORT: ExceptionType = ExceptionType(3);
    /// Data access memory abort
    pub const EXCEPT_ARM_DATA_ABORT: ExceptionType = ExceptionType(4);
    /// Reserved
    pub const EXCEPT_ARM_RESERVED: ExceptionType = ExceptionType(5);
    /// Normal interrupt
    pub const EXCEPT_ARM_IRQ: ExceptionType = ExceptionType(6);
    /// Fast interrupt
    pub const EXCEPT_ARM_FIQ: ExceptionType = ExceptionType(7);
    /// In the UEFI spec for "convenience", unsure if we'll need it. Set to `EXCEPT_ARM_FIQ`
    pub const MAX_ARM_EXCEPTION: ExceptionType = ExceptionType::EXCEPT_ARM_FIQ;
}

#[cfg(target_arch = "aarch64")]
impl ExceptionType {
    /// Synchronous exception, such as attempting to execute an invalid instruction
    pub const EXCEPT_AARCH64_SYNCHRONOUS_EXCEPTIONS: ExceptionType = ExceptionType(0);
    /// Normal interrupt
    pub const EXCEPT_AARCH64_IRQ: ExceptionType = ExceptionType(1);
    /// Fast interrupt
    pub const EXCEPT_AARCH64_FIQ: ExceptionType = ExceptionType(2);
    /// System Error
    pub const EXCEPT_AARCH64_SERROR: ExceptionType = ExceptionType(3);
    /// In the UEFI spec for "convenience", unsure if we'll need it. Set to `EXCEPT_AARCH64_SERROR`
    pub const MAX_AARCH64_EXCEPTION: ExceptionType = ExceptionType::EXCEPT_AARCH64_SERROR;
}

#[cfg(target_arch = "riscv")]
impl ExceptionType {
    /// Instruction misaligned
    pub const EXCEPT_RISCV_INST_MISALIGNED: ExceptionType = ExceptionType(0);
    /// Instruction access fault
    pub const EXCEPT_RISCV_INST_ACCESS_FAULT: ExceptionType = ExceptionType(1);
    /// Illegal instruction
    pub const EXCEPT_RISCV_ILLEGAL_INST: ExceptionType = ExceptionType(2);
    /// Breakpoint
    pub const EXCEPT_RISCV_BREAKPOINT: ExceptionType = ExceptionType(3);
    /// Load address misaligned
    pub const EXCEPT_RISCV_LOAD_ADDRESS_MISALIGNED: ExceptionType = ExceptionType(4);
    /// Load accept fault
    pub const EXCEPT_RISCV_LOAD_ACCESS_FAULT: ExceptionType = ExceptionType(5);
    /// Store AMO address misaligned
    pub const EXCEPT_RISCV_STORE_AMO_ADDRESS_MISALIGNED: ExceptionType = ExceptionType(6);
    /// Store AMO access fault
    pub const EXCEPT_RISCV_STORE_AMO_ACCESS_FAULT: ExceptionType = ExceptionType(7);
    /// ECALL from User mode
    pub const EXCEPT_RISCV_ENV_CALL_FROM_UMODE: ExceptionType = ExceptionType(8);
    /// ECALL from Supervisor mode
    pub const EXCEPT_RISCV_ENV_CALL_FROM_SMODE: ExceptionType = ExceptionType(9);
    /// ECALL from Machine mode
    pub const EXCEPT_RISCV_ENV_CALL_FROM_MMODE: ExceptionType = ExceptionType(11);
    /// Instruction page fault
    pub const EXCEPT_RISCV_INST_PAGE_FAULT: ExceptionType = ExceptionType(12);
    /// Load page fault
    pub const EXCEPT_RISCV_LOAD_PAGE_FAULT: ExceptionType = ExceptionType(13);
    /// Store AMO page fault
    pub const EXCEPT_RISCV_STORE_AMO_PAGE_FAULT: ExceptionType = ExceptionType(15);
    // RISC-V interrupt types
    /// Supervisor software interrupt
    pub const EXCEPT_RISCV_SUPERVISOR_SOFTWARE_INT: ExceptionType = ExceptionType(1);
    /// Machine software interrupt
    pub const EXCEPT_RISCV_MACHINE_SOFTWARE_INT: ExceptionType = ExceptionType(3);
    /// Supervisor timer interrupt
    pub const EXCEPT_RISCV_SUPERVISOR_TIMER_INT: ExceptionType = ExceptionType(5);
    /// Machine timer interrupt
    pub const EXCEPT_RISCV_MACHINE_TIMER_INT: ExceptionType = ExceptionType(7);
    /// Supervisor external interrupt
    pub const EXCEPT_RISCV_SUPERVISOR_EXTERNAL_INT: ExceptionType = ExceptionType(9);
    /// Machine external interrupt
    pub const EXCEPT_RISCV_MACHINE_EXTERNAL_INT: ExceptionType = ExceptionType(11);
}
