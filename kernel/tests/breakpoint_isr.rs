#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(kernel::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel::arch::x86_64::halt;
use kernel::arch::x86_64::interrupts::{int3, InterruptDescriptorTable, InterruptStackFrame};
use kernel::arch::x86_64::segmentation::*;
use lazy_static::lazy_static;

struct FullGdt {
    table: GlobalDescriptorTable,
    kernel_code: SegmentSelector,
}

lazy_static! {
    static ref GDT: FullGdt = {
        let mut table = GlobalDescriptorTable::new();

        let kernel_code = table
            .add_entry(SegmentDescriptor::NormalSegment(NormalSegment::KERNEL_CODE))
            .unwrap();

        FullGdt { table, kernel_code }
    };
}

extern "x86-interrupt" fn debug_handler(_: InterruptStackFrame) {}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let kernel_segment = GDT.kernel_code;
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler(kernel_segment, debug_handler);

        idt
    };
}

entry_point!(_start);
fn _start(_boot_info: &'static BootInfo) -> ! {
    GDT.table.load();
    IDT.load();

    // If the handler is not correctly set then the test will fail.
    int3();

    test_main();

    halt()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::testing::test_panic_handler(info)
}
