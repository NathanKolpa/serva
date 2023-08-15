#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(_start);

fn _start(boot_info: &'static BootInfo) -> ! {
    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::testing::test_panic_handler(info)
}
