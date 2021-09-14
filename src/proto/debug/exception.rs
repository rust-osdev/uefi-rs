/// Represents supported CPU exceptions.
// FIXME: Should probably move this into some sort of enum,
//        but the repeating values might make that difficult.
pub type ExceptionType = isize;

// EBC Exception types
#[allow(dead_code)]
pub const EXCEPT_EBC_UNDEFINED: ExceptionType = 0;
#[allow(dead_code)]
pub const EXCEPT_EBC_DIVIDE_ERROR: ExceptionType = 1;
#[allow(dead_code)]
pub const EXCEPT_EBC_DEBUG: ExceptionType = 2;
#[allow(dead_code)]
pub const EXCEPT_EBC_BREAKPOINT: ExceptionType = 3;
#[allow(dead_code)]
pub const EXCEPT_EBC_OVERFLOW: ExceptionType = 4;
#[allow(dead_code)]
pub const EXCEPT_EBC_INVALID_OPCODE: ExceptionType = 5;
#[allow(dead_code)]
pub const EXCEPT_EBC_STACK_FAULT: ExceptionType = 6;
#[allow(dead_code)]
pub const EXCEPT_EBC_ALIGNMENT_CHECK: ExceptionType = 7;
#[allow(dead_code)]
pub const EXCEPT_EBC_INSTRUCTION_ENCODING: ExceptionType = 8;
#[allow(dead_code)]
pub const EXCEPT_EBC_BAD_BREAK: ExceptionType = 9;
#[allow(dead_code)]
pub const EXCEPT_EBC_SINGLE_STEP: ExceptionType = 10;

// IA-32 Exception types
#[allow(dead_code)]
pub const EXCEPT_IA32_DIVIDE_ERROR: ExceptionType = 0;
#[allow(dead_code)]
pub const EXCEPT_IA32_DEBUG: ExceptionType = 1;
#[allow(dead_code)]
pub const EXCEPT_IA32_NMI: ExceptionType = 2;
#[allow(dead_code)]
pub const EXCEPT_IA32_BREAKPOINT: ExceptionType = 3;
#[allow(dead_code)]
pub const EXCEPT_IA32_OVERFLOW: ExceptionType = 4;
#[allow(dead_code)]
pub const EXCEPT_IA32_BOUND: ExceptionType = 5;
#[allow(dead_code)]
pub const EXCEPT_IA32_INVALID_OPCODE: ExceptionType = 6;
#[allow(dead_code)]
pub const EXCEPT_IA32_DOUBLE_FAULT: ExceptionType = 8;
#[allow(dead_code)]
pub const EXCEPT_IA32_INVALID_TSS: ExceptionType = 10;
#[allow(dead_code)]
pub const EXCEPT_IA32_SEG_NOT_PRESENT: ExceptionType = 11;
#[allow(dead_code)]
pub const EXCEPT_IA32_STACK_FAULT: ExceptionType = 12;
#[allow(dead_code)]
pub const EXCEPT_IA32_GP_FAULT: ExceptionType = 13;
#[allow(dead_code)]
pub const EXCEPT_IA32_PAGE_FAULT: ExceptionType = 14;
#[allow(dead_code)]
pub const EXCEPT_IA32_FP_ERROR: ExceptionType = 16;
#[allow(dead_code)]
pub const EXCEPT_IA32_ALIGNMENT_CHECK: ExceptionType = 17;
#[allow(dead_code)]
pub const EXCEPT_IA32_MACHINE_CHECK: ExceptionType = 18;
#[allow(dead_code)]
pub const EXCEPT_IA32_SIMD: ExceptionType = 19;

// X64 Exception types
#[allow(dead_code)]
pub const EXCEPT_X64_DIVIDE_ERROR: ExceptionType = 0;
#[allow(dead_code)]
pub const EXCEPT_X64_DEBUG: ExceptionType = 1;
#[allow(dead_code)]
pub const EXCEPT_X64_NMI: ExceptionType = 2;
#[allow(dead_code)]
pub const EXCEPT_X64_BREAKPOINT: ExceptionType = 3;
#[allow(dead_code)]
pub const EXCEPT_X64_OVERFLOW: ExceptionType = 4;
#[allow(dead_code)]
pub const EXCEPT_X64_BOUND: ExceptionType = 5;
#[allow(dead_code)]
pub const EXCEPT_X64_INVALID_OPCODE: ExceptionType = 6;
#[allow(dead_code)]
pub const EXCEPT_X64_DOUBLE_FAULT: ExceptionType = 8;
#[allow(dead_code)]
pub const EXCEPT_X64_INVALID_TSS: ExceptionType = 10;
#[allow(dead_code)]
pub const EXCEPT_X64_SEG_NOT_PRESENT: ExceptionType = 11;
#[allow(dead_code)]
pub const EXCEPT_X64_STACK_FAULT: ExceptionType = 12;
#[allow(dead_code)]
pub const EXCEPT_X64_GP_FAULT: ExceptionType = 13;
#[allow(dead_code)]
pub const EXCEPT_X64_PAGE_FAULT: ExceptionType = 14;
#[allow(dead_code)]
pub const EXCEPT_X64_FP_ERROR: ExceptionType = 16;
#[allow(dead_code)]
pub const EXCEPT_X64_ALIGNMENT_CHECK: ExceptionType = 17;
#[allow(dead_code)]
pub const EXCEPT_X64_MACHINE_CHECK: ExceptionType = 18;
#[allow(dead_code)]
pub const EXCEPT_X64_SIMD: ExceptionType = 19;

// TODO: IPF Exception types
// TODO: ARM Exception types
// TODO: Aarch64 Exception types
// TODO: RiscV Exception types
// TODO: RiscV interrupt types
