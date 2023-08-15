use lazy_static::lazy_static;
use crate::arch::x86_64::interrupts::{InterruptDescriptorTable, InterruptStackFrame};

use crate::arch::x86_64::segmentation::*;

const DOUBLE_FAULT_IST_INDEX: usize = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        static mut STACK: [u8; 4096 * 5] = [0; 4096 * 5];
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] =
            InterruptStackRef::from_stack(unsafe { &mut STACK });
        tss
    };
}

struct FullGdt {
    table: GlobalDescriptorTable,
    kernel_code: SegmentSelector,
    tss: SegmentSelector,
}

lazy_static! {
    static ref GDT: FullGdt = {
        let mut table = GlobalDescriptorTable::new();

        let kernel_code = table
            .add_entry(SegmentDescriptor::NormalSegment(
                NormalSegment::KERNEL_CODE64,
            ))
            .unwrap();

        let tss = table
            .add_entry(SegmentDescriptor::LongSystemSegment(LongSegment::new_tss(
                &TSS,
            )))
            .unwrap();

        FullGdt {
            table,
            kernel_code,
            tss,
        }
    };
}

extern "x86-interrupt" fn double_fault_handler(_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("Double fault interrupt")
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let kernel_segment = GDT.kernel_code;

        let mut idt = InterruptDescriptorTable::new();

        idt.double_fault.set_handler(kernel_segment, double_fault_handler);
        idt.double_fault.set_stack_index(DOUBLE_FAULT_IST_INDEX);

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
