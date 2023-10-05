use crate::arch::x86_64::init::GDT;
use crate::multi_tasking::scheduler::stack::ThreadStack;
use crate::service::Id;
use essentials::address::VirtualAddress;
use x86_64::interrupts::context::{InterruptStackFrame, InterruptedContext};
use x86_64::RFlags;

pub type ThreadId = usize;

#[derive(Debug)]
pub enum ThreadState {
    Running,
    Waiting,
    Blocked { next: Option<ThreadId> },
}

#[derive(Debug)]
pub struct Thread {
    #[allow(dead_code)] // impl Debug ignores the usage of this field
    name: Option<&'static str>,
    context: InterruptedContext,
    state: ThreadState,
    service_id: Option<Id>,
}

impl Thread {
    pub unsafe fn start_new(
        name: Option<&'static str>,
        stack: ThreadStack,
        entrypoint: VirtualAddress,
        service_id: Option<Id>,
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
            service_id,
        }
    }

    pub fn finish_tick(&mut self) {
        match self.state {
            ThreadState::Running => self.state = ThreadState::Waiting,
            _ => {}
        }
    }

    pub fn start_tick(&mut self) {
        match self.state {
            ThreadState::Waiting => self.state = ThreadState::Running,
            _ => {}
        }
    }

    pub fn block(&mut self) {
        self.state = ThreadState::Blocked { next: None }
    }

    pub fn unblock(&mut self) -> Option<ThreadId> {
        match self.state {
            ThreadState::Blocked { next } => {
                self.state = ThreadState::Waiting;
                next
            }
            _ => None,
        }
    }

    pub fn set_next_block(&mut self, next_id: ThreadId) {
        match &mut self.state {
            ThreadState::Blocked { next } => {
                *next = Some(next_id);
            }
            _ => {}
        }
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

    pub fn service_id(&self) -> Option<Id> {
        self.service_id
    }
}
