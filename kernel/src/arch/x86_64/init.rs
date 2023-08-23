use crate::arch::x86_64::constants::{MIN_STACK_SIZE, TICK_INTERRUPT_INDEX};
use crate::arch::x86_64::devices::pic_8259::PIC_CHAIN;
use crate::arch::x86_64::interrupts::{
    InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode,
};
use crate::arch::x86_64::segmentation::*;
use crate::arch::x86_64::PrivilegeLevel;
use crate::util::address::VirtualAddress;
use crate::util::Singleton;
use crate::{debug_print, debug_println};

const DOUBLE_FAULT_IST_INDEX: usize = 0;

fn init_tss() -> TaskStateSegment {
    let mut tss = TaskStateSegment::new();
    static mut STACK: [u8; MIN_STACK_SIZE] = [0; MIN_STACK_SIZE];
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] =
        InterruptStackRef::from_slice(unsafe { &mut STACK });

    static mut PSTACK: [u8; MIN_STACK_SIZE] = [0; MIN_STACK_SIZE];

    tss.privilege_stack_table[0] = InterruptStackRef::from_slice(unsafe { &mut PSTACK });

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
    pub syscall: SegmentSelector,
    pub sysret: SegmentSelector,
}

fn init_gdt() -> FullGdt {
    let mut table = GlobalDescriptorTable::new();

    let kernel_code = table.add_entry(SegmentDescriptor::KERNEL_CODE).unwrap();
    // Kernel data is required by syscall to be the next entry after kernel code.
    let kernel_data = table.add_entry(SegmentDescriptor::KERNEL_DATA).unwrap();

    // User data is required by sysret to the next entry after the selector.
    let user_data = table.add_entry(SegmentDescriptor::USER_DATA).unwrap();
    // User code is required by sysret to be the next entry after user data.
    let user_code = table.add_entry(SegmentDescriptor::USER_CODE).unwrap();

    let tss = table.add_entry(SegmentDescriptor::new_tss(&TSS)).unwrap();

    FullGdt {
        table,
        kernel_code,
        kernel_data,
        user_code,
        user_data,
        tss,
        syscall: SegmentSelector::new(kernel_code.index(), PrivilegeLevel::Ring0),
        sysret: SegmentSelector::new(kernel_data.index(), PrivilegeLevel::Ring3),
    }
}

pub static GDT: Singleton<FullGdt> = Singleton::new(init_gdt);

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, error_code: u64) -> ! {
    panic!("Double fault: {error_code} {frame:?}")
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

    let addr = VirtualAddress::from(addr);

    debug_println!("Page fault interrupt at {addr:?} because {error_code:?}")
}

extern "x86-interrupt" fn tick(_: InterruptStackFrame) {
    debug_print!(".");
    PIC_CHAIN
        .lock()
        .end_of_interrupt(TICK_INTERRUPT_INDEX as u8);
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

    idt[TICK_INTERRUPT_INDEX].set_handler(kernel_segment, tick);

    idt
}

pub static IDT: Singleton<InterruptDescriptorTable> = Singleton::new(init_idt);

/// Initialize x86_64-specific components for the kernel.
pub fn init_x86_64() {
    GDT.table.load();

    unsafe {
        GDT.kernel_code.load_into_cs(); // Meaning the current code segment (CS) is the kernel code
        GDT.kernel_data.load_into_ds();
        GDT.tss.load_into_tss(); // Load the TSS.
    }

    IDT.load();

    PIC_CHAIN.lock(); // init pic chain by calling lock()
}
