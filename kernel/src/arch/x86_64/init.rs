use crate::arch::x86_64::interrupts::{
    InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode,
};
use crate::arch::x86_64::segmentation::*;
use crate::util::address::VirtualAddress;
use crate::util::Singleton;

const DOUBLE_FAULT_IST_INDEX: usize = 0;

fn init_tss() -> TaskStateSegment {
    let mut tss = TaskStateSegment::new();
    static mut STACK: [u8; 4096] = [0; 4096];
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] =
        InterruptStackRef::from_slice(unsafe { &mut STACK });

    static mut PRIV_STACK: [u8; 4096] = [0; 4096];

    tss.privilege_stack_table[0] = InterruptStackRef::from_slice(unsafe { &mut PRIV_STACK });

    tss
}

pub static TSS: Singleton<TaskStateSegment> = Singleton::new(init_tss);

pub struct FullGdt {
    pub table: GlobalDescriptorTable,
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
    pub user_code: SegmentSelector,
    pub user_data: SegmentSelector,
    pub tss: SegmentSelector,
}

fn init_gdt() -> FullGdt {
    let mut table = GlobalDescriptorTable::new();

    let kernel_code = table.add_entry(SegmentDescriptor::KERNEL_CODE).unwrap();
    let kernel_data = table.add_entry(SegmentDescriptor::KERNEL_DATA).unwrap();
    let user_data = table.add_entry(SegmentDescriptor::USER_DATA).unwrap();
    let user_code = table.add_entry(SegmentDescriptor::USER_CODE).unwrap();
    let tss = table.add_entry(SegmentDescriptor::new_tss(&TSS)).unwrap();

    FullGdt {
        table,
        kernel_code,
        kernel_data,
        user_code,
        user_data,
        tss,
    }
}

pub static GDT: Singleton<FullGdt> = Singleton::new(init_gdt);

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, error_code: u64) -> ! {
    panic!("Double fault interrupt {error_code} {frame:?}")
}

extern "x86-interrupt" fn general_protection_fault_handler(
    frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("General protection fault interrupt {error_code} {frame:?}")
}

extern "x86-interrupt" fn page_fault_handler(
    _frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let addr: u64;

    unsafe {
        core::arch::asm!("mov {}, cr2", out(reg) addr, options(nomem, nostack, preserves_flags));
    }

    let addr = VirtualAddress::new(addr);

    panic!("Page fault interrupt at {addr:?} because {error_code:?}")
}

fn init_idt() -> InterruptDescriptorTable {
    let kernel_segment = GDT.kernel_code;

    let mut idt = InterruptDescriptorTable::new();

    idt.double_fault
        .set_handler(kernel_segment, double_fault_handler);
    idt.double_fault.set_stack_index(DOUBLE_FAULT_IST_INDEX);

    idt.general_protection_fault
        .set_handler(kernel_segment, general_protection_fault_handler);
    idt.double_fault.set_stack_index(DOUBLE_FAULT_IST_INDEX); // TODO

    idt.page_fault
        .set_handler(kernel_segment, page_fault_handler);
    idt.double_fault.set_stack_index(DOUBLE_FAULT_IST_INDEX); // TODO


    idt
}

pub static IDT: Singleton<InterruptDescriptorTable> = Singleton::new(init_idt);

pub fn init_x86_64() {
    GDT.table.load();

    unsafe {
        GDT.kernel_code.load_into_cs(); // Meaning the current code segment (CS) is the kernel code
        GDT.kernel_data.load_into_ds();
        GDT.tss.load_into_tss(); // Load the TSS.
    }

    IDT.load();
}
