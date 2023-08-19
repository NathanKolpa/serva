use bootloader::BootInfo;

use crate::arch::x86_64::{halt, init_x86_64, ARCH_NAME};
use crate::arch::x86_64::paging::PhysicalPage;
use crate::debug::DEBUG_CHANNEL;
use crate::memory::{FRAME_ALLOCATOR, MemoryMapper};

pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    debug_println!("Starting the Serva Operating System...");
    debug_println!("Architecture: {ARCH_NAME}");
    debug_println!("Debug channel: {DEBUG_CHANNEL}");

    init_x86_64();

    let memory_mapper = unsafe {
        FRAME_ALLOCATOR.init(&boot_info.memory_map);
        MemoryMapper::new(&FRAME_ALLOCATOR, PhysicalPage::active().0, boot_info.physical_memory_offset)
    };

    debug_println!("{:#?}", FRAME_ALLOCATOR.info());

    halt()
}
