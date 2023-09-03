use core::panic::PanicInfo;

use bootloader::BootInfo;

use crate::arch::x86_64::paging::PhysicalPage;
use crate::arch::x86_64::ARCH_NAME;
use crate::arch::x86_64::{halt_loop, init_x86_64};
use crate::debug::DEBUG_CHANNEL;
use crate::interrupts::INTERRUPT_HANDLERS;
use crate::memory::{MemoryMapper, FRAME_ALLOCATOR};

/// The kernel panic handler.
pub fn handle_panic(info: &PanicInfo) -> ! {
    debug_println!("{info}");
    halt_loop()
}

/// The kernel entry point.
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    debug_println!("Starting the Serva Operating System...");
    debug_println!("Architecture: {ARCH_NAME}");
    debug_println!("Debug channel: {DEBUG_CHANNEL}");

    init_x86_64(INTERRUPT_HANDLERS);

    let _memory_mapper = unsafe {
        FRAME_ALLOCATOR.init(&boot_info.memory_map);
        MemoryMapper::new(
            &FRAME_ALLOCATOR,
            PhysicalPage::active().0,
            boot_info.physical_memory_offset,
        )
    };

    debug_println!("{:#?}", FRAME_ALLOCATOR.info());

    halt_loop()
}
