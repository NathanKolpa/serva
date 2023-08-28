use core::arch::asm;

use crate::arch::x86_64::interrupts::InterruptStackFrame;
use crate::debug_println;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct RegisterContext {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    pub rbp: u64,
}

impl RegisterContext {
    #[inline(always)]
    pub fn current() -> Self {
        // good god...
        let r15: u64;
        let r14: u64;
        let r13: u64;
        let r12: u64;
        let r11: u64;
        let r10: u64;
        let r9: u64;
        let r8: u64;
        let rdi: u64;
        let rsi: u64;
        let rdx: u64;
        let rcx: u64;
        let rbx: u64;
        let rax: u64;
        let rbp: u64;

        unsafe {
            asm!(
                "mov {rbx}, rbx",
                "mov {rbp}, rbp",
                out("r15") r15,
                out("r14") r14,
                out("r13") r13,
                out("r12") r12,
                out("r11") r11,
                out("r10") r10,
                out("r9") r9,
                out("r8") r8,
                out("rdi") rdi,
                out("rsi") rsi,
                out("rdx") rdx,
                out("rcx") rcx,
                rbx = out(reg) rbx,
                out("rax") rax,
                rbp = out(reg) rbp,
                options(nostack, nomem)
            )
        }

        Self {
            r15,
            r14,
            r13,
            r12,
            r11,
            r10,
            r9,
            r8,
            rdi,
            rsi,
            rdx,
            rcx,
            rbx,
            rax,
            rbp,
        }
    }

    #[inline(always)]
    pub unsafe fn restore(&self) {
        asm!(
            "",
            "mov rbx, {rbx}",
            "mov rbp, {rbp}",
            in("r15") self.r15,
            in("r14") self.r14,
            in("r13") self.r13,
            in("r12") self.r12,
            in("r11") self.r11,
            in("r10") self.r10,
            in("r9") self.r9,
            in("r8") self.r8,
            in("rdi") self.rdi,
            in("rsi") self.rsi,
            in("rdx") self.rdx,
            in("rcx") self.rcx,
            rbx = in(reg) self.rbx,
            in("rax") self.rax,
            rbp = in(reg) self.rbp,
            options(nostack)
        )
    }
}

#[derive(Clone, Debug)]
pub struct InterruptedContext {
    pub interrupt_stack_frame: InterruptStackFrame,
    pub registers: RegisterContext,
}

impl InterruptedContext {
    #[inline(always)]
    pub fn current(interrupt_stack_frame: InterruptStackFrame) -> Self {
        let registers = RegisterContext::current();

        Self {
            registers,
            interrupt_stack_frame,
        }
    }

    pub unsafe fn restore(
        &self,
    ) -> ! {
        self.registers.restore();
        self.interrupt_stack_frame.iretq();
    }
}
