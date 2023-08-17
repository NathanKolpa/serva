use lazy_static::lazy_static;

use crate::arch::x86_64::interrupts::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::arch::x86_64::segmentation::*;
use crate::arch::x86_64::trampoline::enter_ring3;

const DOUBLE_FAULT_IST_INDEX: usize = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        static mut STACK: [u8; 4096 * 5] = [0; 4096 * 5];
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] =
            InterruptStackRef::from_slice(unsafe { &mut STACK });
        tss
    };
}

pub struct FullGdt {
    pub table: GlobalDescriptorTable,
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
    pub user_code: SegmentSelector,
    pub user_data: SegmentSelector,
    pub tss: SegmentSelector,
}

lazy_static! {
    pub static ref GDT: FullGdt = {
        let mut table = GlobalDescriptorTable::new();

        let kernel_code = table.add_entry(SegmentDescriptor::KERNEL_CODE).unwrap();
        let kernel_data = table.add_entry(SegmentDescriptor::KERNEL_DATA).unwrap();
        let tss = table.add_entry(SegmentDescriptor::new_tss(&TSS)).unwrap();
        let user_code = table.add_entry(SegmentDescriptor::USER_CODE).unwrap();
        let user_data = table.add_entry(SegmentDescriptor::USER_CODE).unwrap();

        FullGdt {
            table,
            kernel_code,
            kernel_data,
            user_code,
            user_data,
            tss,
        }
    };
}

extern "x86-interrupt" fn double_fault_handler(_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("Double fault interrupt")
}

extern "x86-interrupt" fn general_protection_fault_handler(
    _frame: InterruptStackFrame,
    _error_code: u64,
) {
    panic!("General protection fault interrupt")
}

extern "x86-interrupt" fn page_fault_handler(_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    panic!("Page fault interrupt {error_code:?}")
}

extern "x86-interrupt" fn segment_not_present_handler(_frame: InterruptStackFrame) {
    panic!("Segment not present interrupt")
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
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

        idt.stack_segment_fault
            .set_handler(kernel_segment, segment_not_present_handler);
        idt.stack_segment_fault.set_stack_index(DOUBLE_FAULT_IST_INDEX); // TODO

        idt
    };
}

pub fn init_x86_64() {
    GDT.table.load();

    unsafe {
        GDT.kernel_code.load_into_cs(); // Meaning the current code segment (CS) is the kernel code
        GDT.tss.load_into_tss(); // Load the TSS.
    }

    IDT.load();
}
