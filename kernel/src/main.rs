#![no_std]
#![no_main]

extern crate bootloader;

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}

entry_point!(_start);

fn _start(boot_info: &'static BootInfo) -> ! {
    loop {}
}
