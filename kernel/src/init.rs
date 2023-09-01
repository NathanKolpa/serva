use core::panic::PanicInfo;

use bootloader::BootInfo;

use crate::arch::x86_64::init::GDT;
use crate::arch::x86_64::paging::PhysicalPage;
use crate::arch::x86_64::syscalls::{init_syscalls, SyscallArgs};
use crate::arch::x86_64::{halt, halt_loop, init_x86_64, ARCH_NAME};
use crate::arch::x86_64::interrupts::atomic_block;
use crate::debug::DEBUG_CHANNEL;
use crate::interrupts::INTERRUPT_HANDLERS;
use crate::memory::{MemoryMapper, FRAME_ALLOCATOR};
use crate::multi_tasking::scheduler::{ThreadStack, SCHEDULER};
use crate::multi_tasking::sync::Mutex;

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

    let memory_mapper = unsafe {
        FRAME_ALLOCATOR.init(&boot_info.memory_map);
        MemoryMapper::new(
            &FRAME_ALLOCATOR,
            PhysicalPage::active().0,
            boot_info.physical_memory_offset,
        )
    };

    debug_println!("{:#?}", FRAME_ALLOCATOR.info());

    SCHEDULER.initialize(memory_mapper, exit);

    test_problem();
    debug_println!("Initialized the kernel, calling the first scheduler task.");

    unsafe { SCHEDULER.start() }
}

fn exit() -> ! {
    debug_println!("Not tasks left to execute, goodnight!");
    halt_loop()
}

fn test_problem() {
    static mut STACK1: [u8; 1000] = [0; 1000];
    SCHEDULER.new_kernel_thread(unsafe { ThreadStack::from_slice(&mut STACK1) }, thread_1);

    static mut STACK2: [u8; 1000] = [0; 1000];
    SCHEDULER.new_kernel_thread(unsafe { ThreadStack::from_slice(&mut STACK2) }, thread_2);
}

fn thread_1() -> ! {
    loop {
        debug_println!("#1");
        SCHEDULER.yield_current();
    }
}

fn thread_2() -> ! {
    loop {
        debug_println!("#2");
        SCHEDULER.yield_current();
    }
}