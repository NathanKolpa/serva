use crate::arch::x86_64::init::InterruptHandlers;
use crate::arch::x86_64::interrupts::context::InterruptedContext;
use crate::multi_tasking::scheduler::SCHEDULER;

fn tick(ctx: InterruptedContext) -> &'static InterruptedContext {
    SCHEDULER.get_next_context(ctx)
}

pub const INTERRUPT_HANDLERS: InterruptHandlers = InterruptHandlers { tick };
