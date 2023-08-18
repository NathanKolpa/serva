use core::arch::asm;

use crate::arch::x86_64::segmentation::SegmentSelector;
use crate::util::address::VirtualAddress;

pub unsafe fn enter_ring3(
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
        "push 0x200",           // rflags (only interrupt bit set)
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
