#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

use kernel::init::{handle_panic, kernel_main};

entry_point!(_start);

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    handle_panic(info)
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::testing::test_panic_handler(info)
}

fn _start(boot_info: &'static BootInfo) -> ! {
    #[cfg(test)]
    test_main();

    kernel_main(boot_info)
}
