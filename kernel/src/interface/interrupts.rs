use crate::arch::x86_64::init::InterruptHandlers;
use crate::arch::x86_64::interrupts::context::InterruptedContext;
use crate::multi_tasking::scheduler::SCHEDULER;
use crate::service::SERVICE_TABLE;

fn tick(ctx: InterruptedContext) -> *const InterruptedContext {
    let (ctx, service) = SCHEDULER.tick(ctx);

    if let Some(service) = service {
        service.set_memory_map_active();
    }

    ctx
}

pub const INTERRUPT_HANDLERS: InterruptHandlers = InterruptHandlers { tick };
