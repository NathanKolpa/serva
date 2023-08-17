use core::arch::asm;

use crate::arch::x86_64::segmentation::SegmentSelector;
use crate::util::address::VirtualAddress;

pub fn enter_ring3(
    code: VirtualAddress,
    stack_end: VirtualAddress,
    code_segment: SegmentSelector,
    data_segment: SegmentSelector,
) -> ! {
    let code_addr = code.as_u64();
    let stack_addr = stack_end.as_u64();
    let code_segment = code_segment.as_u16();
    let data_segment = data_segment.as_u16();

    unsafe {
        asm!(
            "push {data_segment:x}",  // stack segment
            "push {stack_addr}",    // rsp
            "push 0x200",           // rflags (only interrupt bit set)
            "push {code_segment:x}",   // code segment
            "push {code_addr}",      // ret to virtual addr
            "iretq",
            code_addr = in(reg) code_addr,
            stack_addr = in(reg) stack_addr,
            code_segment = in(reg) code_segment,
            data_segment = in(reg) data_segment,
            options(noreturn)
        )
    }
}
