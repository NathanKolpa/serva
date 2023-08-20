//! Instructions and routines related to changing the CS register.

use core::arch::asm;

use crate::arch::x86_64::segmentation::SegmentSelector;
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
) -> ! {
    let code_addr = code.as_u64();
    let stack_addr = stack_end.as_u64();
    let code_segment = code_segment.as_u16();
    let data_segment = data_segment.as_u16();

    asm!(
        "push rax",  // stack segment
        "push rsi",    // rsp
        "pushf",           // rflags (only interrupt bit set)
        "push rdx",   // code segment
        "push rdi",      // ret to virtual addr
        "iretq",
        in("rdi") code_addr,
        in("rsi") stack_addr,
        in("dx") code_segment,
        in("ax") data_segment,
        options(noreturn)
    )
}

#[naked]
fn handle_syscall() {
    unsafe {
        asm!("sysretq", options(noreturn));
    }
}

pub unsafe fn init_syscall() {
    let handler_addr = handle_syscall as *const () as u64;
    const MSR_LSTAR: u64 = 0xc0000082;
    const MSR_FMASK: u64 = 0xc0000084;
    const MSR_STAR: u64 = 0xC0000081;

    // clear Interrupt flag on syscall with AMD's MSR_FSTAR register
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
        "mov rdx, 0x230008", // TODO: figure out how to dynamically use the segment selectors
        "wrmsr",
        in("rcx") MSR_STAR,
        out("rax") _,
        out("rdx") _
    )
}
