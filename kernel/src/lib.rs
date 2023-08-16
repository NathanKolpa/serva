#![no_std]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![cfg_attr(test, no_main)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use crate::arch::x86_64::halt;
use crate::init::kernel_main;

pub mod arch;
pub mod debug;
pub mod devices;
pub mod init;
#[cfg(not)]
pub mod memory;
pub mod testing;
pub mod util;

#[no_mangle]
pub extern "C" fn _kernel_start() -> ! {
    #[cfg(test)]
    test_main();
    kernel_main()
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use crate::arch::x86_64::halt;

    debug_println!("Kernel Panic: {info}");
    halt()
}
