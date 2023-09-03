use crate::arch::x86_64::init::GDT;
use crate::arch::x86_64::interrupts::context::{InterruptStackFrame, InterruptedContext};
use crate::arch::x86_64::RFlags;
use crate::multi_tasking::scheduler::stack::ThreadStack;
use crate::util::address::VirtualAddress;

pub type ThreadId = usize;

pub enum ThreadState {
    Running,
    Waiting,
    Blocked { next: Option<ThreadId> },
}

pub struct Thread {
    name: Option<&'static str>,
    context: InterruptedContext,
    state: ThreadState,
}

impl Thread {
    pub unsafe fn start_new(
        name: Option<&'static str>,
        stack: ThreadStack,
        entrypoint: VirtualAddress,
    ) -> Self {
        Self {
            name,
            context: InterruptedContext::start_new(InterruptStackFrame::new(
                entrypoint,
                stack.top(),
                RFlags::INTERRUPTS_ENABLED,
                GDT.kernel_code,
                GDT.kernel_data,
            )),
            state: ThreadState::Waiting,
        }
    }

    pub fn set_state(&mut self, state: ThreadState) {
        self.state = state;
    }

    pub fn can_run(&self) -> bool {
        match self.state {
            ThreadState::Running => false,
            ThreadState::Waiting => true,
            ThreadState::Blocked { .. } => false,
        }
    }

    pub fn save(&mut self, ctx: InterruptedContext) {
        self.context = ctx;
    }

    pub fn context_ptr(&self) -> *const InterruptedContext {
        &self.context
    }
}
