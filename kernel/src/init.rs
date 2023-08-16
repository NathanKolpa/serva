use crate::arch::x86_64::{ARCH_NAME, halt, init_x86_64};
use crate::debug::DEBUG_CHANNEL;

pub fn kernel_main() -> ! {
    init_x86_64();

    /*
    debug_println!("Starting the Serva Operating System...");
    debug_println!("Architecture: {ARCH_NAME}");
    debug_println!("Debug channel: {DEBUG_CHANNEL}");

    */
    #[cfg(not)]
    unsafe {
        init_memory_mapper(boot_info);
    }

    #[cfg(not)]
    debug_println!("{:#?}", MEMORY_MAPPER.read().info());

    halt()
}
