use crate::arch::x86_64::interrupts::context::InterruptedContext;
use crate::arch::x86_64::init::InterruptHandlers;
use crate::multi_tasking::scheduler::SCHEDULER;

unsafe fn tick(ctx: *const InterruptedContext) -> *const InterruptedContext {
    SCHEDULER.next_context(ctx)
}

pub const INTERRUPT_HANDLERS: InterruptHandlers = InterruptHandlers { tick };
