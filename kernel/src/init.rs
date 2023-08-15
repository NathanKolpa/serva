use bootloader::BootInfo;

use crate::arch::x86_64::{halt, init_x86_64};

pub fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    debug_println!("Starting the Serva Operating System...");

    init_x86_64();

    halt()
}
