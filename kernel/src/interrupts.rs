use crate::arch::x86_64::init::InterruptHandlers;
use crate::arch::x86_64::interrupts::context::InterruptedContext;

fn tick(_ctx: InterruptedContext) -> &'static InterruptedContext {
    todo!()
}

pub const INTERRUPT_HANDLERS: InterruptHandlers = InterruptHandlers { tick };
