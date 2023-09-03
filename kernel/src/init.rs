use core::panic::PanicInfo;

use bootloader::BootInfo;

use crate::arch::x86_64::constants::MIN_STACK_SIZE;
use crate::arch::x86_64::interrupts::atomic_block;
use crate::arch::x86_64::paging::PhysicalPage;
use crate::arch::x86_64::ARCH_NAME;
use crate::arch::x86_64::{halt_loop, init_x86_64};
use crate::debug::DEBUG_CHANNEL;
use crate::interrupts::INTERRUPT_HANDLERS;
use crate::memory::{MemoryMapper, FRAME_ALLOCATOR};
use crate::multi_tasking::scheduler::{Thread, ThreadStack, SCHEDULER};
use crate::util::address::VirtualAddress;

/// The kernel panic handler.
pub fn handle_panic(info: &PanicInfo) -> ! {
    debug_println!("{info}");
    halt_loop()
}

/// The kernel entry point.
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    init_x86_64(INTERRUPT_HANDLERS);

    let _memory_mapper = unsafe {
        FRAME_ALLOCATOR.init(&boot_info.memory_map);
        MemoryMapper::new(
            &FRAME_ALLOCATOR,
            PhysicalPage::active().0,
            boot_info.physical_memory_offset,
        )
    };

    SCHEDULER.add_thread(unsafe {
        Thread::start_new(
            Some("Kernel Main"),
            ThreadStack::from_slice(&mut KERNEL_MAIN_STACK),
            VirtualAddress::from(main_kernel_thread as *const fn()),
        )
    });

    SCHEDULER.yield_current()
}

static mut KERNEL_MAIN_STACK: [u8; MIN_STACK_SIZE] = [0; MIN_STACK_SIZE];

/// The main kernel thread.
///
/// Execution is given to this code asap in the init process.
/// This is preferred because `multi_tasking::sync` primitives are allowed only from a scheduled thread.
/// This thread also acts as the idle task.
fn main_kernel_thread() -> ! {
    atomic_block(|| {
        debug_println!("Starting the Serva Operating System...");
        debug_println!("Architecture: {}", ARCH_NAME);
        debug_println!("Debug channel: {}", DEBUG_CHANNEL);
        debug_println!("{:#?}", FRAME_ALLOCATOR.info());
    });

    halt_loop()
}
