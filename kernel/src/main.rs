#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

use kernel::arch::x86_64::halt;
use kernel::debug_println;

entry_point!(_start);

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    debug_println!("Kernel Panic: {}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::testing::test_panic_handler(info)
}

fn _start(_boot_info: &'static BootInfo) -> ! {
    #[cfg(test)]
    test_main();

    debug_println!("Starting the Serva Operating System...");

    halt()
}
