use core::arch::asm;

use crate::arch::x86_64::segmentation::SegmentSelector;
use crate::arch::x86_64::RFlags;
use crate::debug_println;
use crate::util::address::VirtualAddress;

#[repr(C)]
#[derive(Clone, Debug, Default)]
pub struct RegisterContext {
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rbp: usize,
    pub rsi: usize,
    pub rdi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rbx: usize,
    pub rax: usize,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct InterruptStackFrame {
    pub instruction_pointer: VirtualAddress,
    pub code_segment: u64,
    pub cpu_flags: RFlags,
    pub stack_pointer: VirtualAddress,
    pub stack_segment: u64,
}

impl InterruptStackFrame {
    pub fn new(
        instruction_pointer: VirtualAddress,
        stack_pointer: VirtualAddress,
        cpu_flags: RFlags,
        code_segment: SegmentSelector,
        stack_segment: SegmentSelector,
    ) -> Self {
        Self {
            instruction_pointer,
            code_segment: code_segment.as_u16() as u64,
            cpu_flags,
            stack_pointer,
            stack_segment: stack_segment.as_u16() as u64,
        }
    }

    #[inline(always)]
    pub unsafe fn iretq(&self) -> ! {
        asm!(
        "push {data_segment}",
        "push {stack_end}",
        "push {rflags}",
        "push {code_segment}",
        "push {code}",
        "iretq",
        rflags = in(reg) self.cpu_flags.as_u64(),
        code = in(reg) self.instruction_pointer.as_u64(),
        stack_end = in(reg) self.stack_pointer.as_u64(),
        code_segment = in(reg) self.code_segment,
        data_segment = in(reg) self.stack_segment,
        options(noreturn)
        )
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct InterruptedContext {
    pub registers: RegisterContext,
    pub interrupt_stack_frame: InterruptStackFrame,
}

impl InterruptedContext {
    pub fn start_new(interrupt_stack_frame: InterruptStackFrame) -> Self {
        Self {
            registers: Default::default(),
            interrupt_stack_frame,
        }
    }
}
