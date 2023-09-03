use core::fmt::Write;
use core::panic::PanicInfo;

use bootloader::BootInfo;

use crate::arch::x86_64::init::GDT;
use crate::arch::x86_64::interrupts::context::{InterruptStackFrame, InterruptedContext};
use crate::arch::x86_64::paging::PhysicalPage;
use crate::arch::x86_64::{halt, halt_loop, init_x86_64, RFlags};
use crate::interrupts::INTERRUPT_HANDLERS;
use crate::arch::x86_64::ARCH_NAME;
use crate::arch::x86_64::devices::uart_16550::SERIAL;
use crate::arch::x86_64::interrupts::{enable_interrupts, int3};
use crate::debug::DEBUG_CHANNEL;
use crate::memory::{MemoryMapper, FRAME_ALLOCATOR};
use crate::multi_tasking::scheduler::{ThreadStack, SCHEDULER};
use crate::multi_tasking::sync::Mutex;
use crate::util::address::VirtualAddress;

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

static mut STACK1: [u8; 4096] = [0; 4096];
static mut STACK2: [u8; 4096] = [0; 4096];
static mut STACK3: [u8; 4096] = [0; 4096];


fn test_problem() {
    SCHEDULER.new_kernel_thread(unsafe { ThreadStack::from_slice(&mut STACK1) }, thread_1);
    SCHEDULER.new_kernel_thread(unsafe { ThreadStack::from_slice(&mut STACK2) }, thread_2);
    // SCHEDULER.new_kernel_thread(unsafe { ThreadStack::from_slice(&mut STACK3) }, thread_3);
}

static TEST_MUTEX: Mutex<i32> = Mutex::new(0);

fn thread_1() -> ! {
    debug_println!("#1 Init. RFlags {}", RFlags::read().as_u64());

    loop {
        debug_println!("#1 Try to acquire lock. RFlags {}", RFlags::read().as_u64());
        let mut lock = TEST_MUTEX.lock();

        debug_println!("#1 Acquired lock, with value {}. Yielding... RFlags {}", *lock, RFlags::read().as_u64());

        SCHEDULER.yield_current();

        *lock += 1;

        drop(lock);
        debug_println!("#1 Dropped lock. Yielding... RFlags {}", RFlags::read().as_u64());

        SCHEDULER.yield_current();

    }
}

fn thread_2() -> ! {
    debug_println!("#2 Init. RFlags {}", RFlags::read().as_u64());

    loop {
        debug_println!("#2 Try to acquire lock. RFlags {}", RFlags::read().as_u64());
        let mut lock = TEST_MUTEX.lock();


        debug_println!("#2 Acquired lock, with value {}. Yielding... RFlags {}", *lock, RFlags::read().as_u64());

        SCHEDULER.yield_current();

        *lock += 1;

        drop(lock);
        debug_println!("#2 Dropped lock. Yielding... RFlags {}", RFlags::read().as_u64());

        SCHEDULER.yield_current();
    }
}

// TODO: probleem is denk ik dat je halt wel verder moet maar de CPU flags niet worden geset

fn thread_3() -> ! {
    debug_println!("#3 Init. RFlags {}", RFlags::read().as_u64());

    loop {
        debug_println!("#3 Try to acquire lock. RFlags {}", RFlags::read().as_u64());
        let mut lock = TEST_MUTEX.lock();


        debug_println!("#3 Acquired lock, with value {}. Yielding... RFlags {}", *lock, RFlags::read().as_u64());

        SCHEDULER.yield_current();

        *lock += 1;

        drop(lock);
        debug_println!("#3 Dropped lock. Yielding... RFlags {}", RFlags::read().as_u64());

        SCHEDULER.yield_current();
    }
}