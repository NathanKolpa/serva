use crate::arch::x86_64::context::InterruptedContext;
use crate::arch::x86_64::init::InterruptHandlers;
use crate::multi_tasking::scheduler::SCHEDULER;

fn tick(ctx: InterruptedContext) -> ! {
    unsafe { SCHEDULER.next_tick(ctx) }
}

pub const INTERRUPT_HANDLERS: InterruptHandlers = InterruptHandlers { tick };
