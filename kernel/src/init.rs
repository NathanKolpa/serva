use core::fmt::Write;
use core::panic::PanicInfo;

use bootloader::BootInfo;

use crate::arch::x86_64::init::GDT;
use crate::arch::x86_64::interrupts::context::{InterruptStackFrame, InterruptedContext};
use crate::arch::x86_64::paging::PhysicalPage;
use crate::arch::x86_64::{halt_loop, init_x86_64, RFlags};
use crate::interrupts::INTERRUPT_HANDLERS;
use crate::arch::x86_64::ARCH_NAME;
use crate::arch::x86_64::devices::uart_16550::SERIAL;
use crate::arch::x86_64::interrupts::int3;
use crate::debug::DEBUG_CHANNEL;
use crate::memory::{MemoryMapper, FRAME_ALLOCATOR};
use crate::multi_tasking::scheduler::{ThreadStack, SCHEDULER};
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

    int3();

    // SCHEDULER.initialize(memory_mapper, exit);

    // test_problem();
    debug_println!("Initialized the kernel, calling the first scheduler task.");

    // unsafe { SCHEDULER.start() }
    halt_loop()
}

fn exit() -> ! {
    debug_println!("Not tasks left to execute, goodnight!");
    halt_loop()
}

static mut CURRENT_TASK: usize = 0;
static mut TASKS: [Option<InterruptedContext>; 2] = [None, None];


static mut STACK1: [u8; 1000 * 4] = [0; 1000 * 4];
static mut STACK2: [u8; 1000 * 4] = [0; 1000 * 4];

unsafe fn ctx_from_index(index: usize) -> InterruptedContext {
    match index {
        0 => InterruptedContext::start_new(InterruptStackFrame::new(
            VirtualAddress::from(thread_1 as usize),
            VirtualAddress::from(STACK1.as_ptr()) + STACK1.len(),
            RFlags::NONE,
            GDT.kernel_code,
            GDT.kernel_data,
        )),
        1 => InterruptedContext::start_new(InterruptStackFrame::new(
            VirtualAddress::from(thread_2 as usize),
            VirtualAddress::from(STACK2.as_ptr()) + STACK2.len(),
            RFlags::NONE,
            GDT.kernel_code,
            GDT.kernel_data,
        )),
        _ => panic!("Index larger then 1"),
    }
}

pub fn test_tick_handler(ctx: InterruptedContext) -> &'static InterruptedContext {
    let current = unsafe { CURRENT_TASK };

    let current_task = unsafe { &mut TASKS[current] };

    if current_task.is_some() {
        *current_task = Some(ctx);
    }

    let next = unsafe {
        CURRENT_TASK = (CURRENT_TASK + 1) % 2;
        CURRENT_TASK
    };

    let next_task =  unsafe { &mut TASKS[next] };

    if next_task.is_none() {
        *next_task = unsafe { Some(ctx_from_index(next)) };
    }

    next_task.as_ref().unwrap()
}

fn test_problem() {
    // SCHEDULER.new_kernel_thread(unsafe { ThreadStack::from_slice(&mut STACK1) }, thread_1);
    //
    // SCHEDULER.new_kernel_thread(unsafe { ThreadStack::from_slice(&mut STACK2) }, thread_2);
}

fn thread_1() -> ! {
    let mut count = 0;

    loop {
        debug_println!("#1 | c = {count}");
        count += 1;
        SCHEDULER.yield_current();
    }
}

fn thread_2() -> ! {
    let mut count = 0;

    loop {
        debug_println!("#2 | c = {count}");
        count += 1;
        SCHEDULER.yield_current();
    }
}
