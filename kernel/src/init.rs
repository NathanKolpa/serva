use bootloader::BootInfo;

use crate::arch::x86_64::{ARCH_NAME, halt, init_x86_64};
use crate::debug::DEBUG_CHANNEL;
use crate::memory::{init_memory_mapper, MEMORY_MAPPER};

pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    debug_println!("Starting the Serva Operating System...");
    debug_println!("Architecture: {ARCH_NAME}");
    debug_println!("Debug channel: {DEBUG_CHANNEL}");

    init_x86_64();

    unsafe {
        init_memory_mapper(boot_info);
    }

    debug_println!("{:#?}", MEMORY_MAPPER.read().info());

    halt()
}
