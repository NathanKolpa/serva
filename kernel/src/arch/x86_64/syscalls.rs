use core::arch::asm;
use core::mem::MaybeUninit;
use core::ops::Add;

use crate::arch::x86_64::interrupts::{disable_interrupts, enable_interrupts};
use crate::arch::x86_64::segmentation::SegmentSelector;
use crate::arch::x86_64::RFlags;
use crate::debug_println;
use crate::util::address::VirtualAddress;

/// Call the [`iretq`](https://www.felixcloutier.com/x86/iret:iretd:iretq) (64 bit variant of `iret`) instruction, a.k.a. "return from interrupt".
/// According to the [osdev wiki](https://wiki.osdev.org/Getting_to_Ring_3#iret_method), it's not actually required to be in a interrupt handler to call this instruction as the name would otherwise suggest.
/// Meaning that this function can (and should) be used to change the current privilege level.
///
/// ## Parameters
/// - `code` The code that will executed after this function.
/// - `stack_end` The stack address that will be used.
/// - `code_segment` The value that will be stored in the `CS` register.
/// - `data_segment` The value that will be stored in the `DS` register.
///
/// ## Safety
/// You can really mess with the computer if not used correctly, so be careful, I guess.
pub unsafe fn return_from_interrupt(
    code: VirtualAddress,
    stack_end: VirtualAddress,
    code_segment: SegmentSelector,
    data_segment: SegmentSelector,
    rflags: RFlags,
) -> ! {
    asm!(
        "push rax",  // stack segment
        "push rsi",    // rsp
        "push {rflags}",           // rflags
        "push rdx",   // code segment
        "push rdi",      // ret to virtual addr
        "iretq",
        rflags = in(reg) rflags.as_u64(),
        in("rdi") code.as_u64(),
        in("rsi") stack_end.as_u64(),
        in("dx") code_segment.as_u16(),
        in("ax") data_segment.as_u16(),
        options(noreturn)
    )
}

#[derive(Debug)]
pub struct SyscallArgs {
    pub syscall: u64,
    pub arg0: u64,
    pub arg1: u64,
    pub arg2: u64,
    pub arg3: u64,
}

static mut SYSCALL_HANDLER: MaybeUninit<fn(SyscallArgs) -> u64> = MaybeUninit::uninit();

#[no_mangle]
unsafe extern "C" fn syscall_handler() {
    let syscall: u64;
    let arg0: u64;
    let arg1: u64;
    let arg2: u64;
    let arg3: u64;

    asm!(
        "",
        out("rax") syscall,
        out("rdi") arg0,
        out("rsi") arg1,
        out("rdx") arg2,
        out("r10") arg3,
        options()
    );

    let args = SyscallArgs {
        syscall,
        arg0,
        arg1,
        arg2,
        arg3,
    };

    let return_value = (SYSCALL_HANDLER.assume_init_ref())(args);

    asm!(
        "mov rax, {return_value}",
        return_value = in(reg) return_value
    );
}

#[naked]
fn naked_syscall_handler() {
    unsafe {
        asm!(
            // save for return
            "push rcx",
            "push r11",
            // callee saved registers
            "push rbp",
            "push rbx",
            "push r12",
            "push r13",
            "push r14",
            "push r15",
            // call the handler
            "call syscall_handler",
            // restore callee registers
            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop rbx",
            "pop rbp",
            // return
            "pop r11",
            "pop rcx",
            "sysretq",
            options(noreturn)
        );
    }
}

/// Enable and setup the `syscall` instruction with a global handler.
///
/// ## Safety
///
/// The caller must ensure that this function is only called once.
///
/// ## Parameters
/// - `syscall_handler` The function that is called when `syscall` is executed.
/// - `syscall_selector` The GDT selectors that are set active on the syscall.
///     - The index must point to the `syscall_handler`'s code segment (most likely Ring0).
///     - The segment after the code segment the GDT must point to the data segment of the same privilege level.
///     - The privilege level must be consistent with both segments.
/// - `sysret_selector` The GDT selectors that are set active when the `syscall_handler` returns.
///     - The index should point to the **data** segment of the syscallee's **data** segment, **minus one**.
///     - The segment after the data segment in the GDT must point to the code segment of the same privilege level.
///     - The privilege level must be consistent with both segments.
///
/// Be sure to pay extra attention to the selector parameters, because they are very inconsistent with the rest of the architecture and themselves.
pub unsafe fn init_syscalls(
    syscall_handler: fn(SyscallArgs) -> u64,
    syscall_selector: SegmentSelector,
    sysret_selector: SegmentSelector,
) {
    SYSCALL_HANDLER.write(syscall_handler);

    let handler_addr = naked_syscall_handler as *const () as u64;
    const MSR_LSTAR: u64 = 0xc0000082;
    const MSR_FMASK: u64 = 0xc0000084;
    const MSR_STAR: u64 = 0xC0000081;
    const MSR_EFER: u64 = 0xC0000080;

    let mut selector_value: u64 = 0;
    selector_value |= syscall_selector.as_u16() as u64;
    selector_value |= (sysret_selector.as_u16() as u64) << 16;

    // clear the interrupt flag on syscall, this will prevent interrupts from messing with the stack.
    asm!(
        "xor rdx, rdx",
        "mov rax, 0x200",
        "wrmsr",
        in("rcx") MSR_FMASK,
        out("rdx") _
    );

    // write handler address to AMD's MSR_LSTAR register
    asm!(
        "mov rdx, rax",
        "shr rdx, 32",
        "wrmsr",
        in("rax") handler_addr,
        in("rcx") MSR_LSTAR,
        out("rdx") _
    );

    asm!(
        "xor rax, rax",
        "mov rdx, {selector_value:r}", // TODO: figure out how to dynamically use the segment selectors
        "wrmsr",
        in("rcx") MSR_STAR,
        selector_value = in(reg) selector_value,
        out("rax") _,
        out("rdx") _
    );

    // Enable the use of syscall and sysret instructions (disabled by default on boot).
    asm!(
        "rdmsr",
        "or eax, 1",
        "wrmsr",
        in("ecx") MSR_EFER
    );
}
