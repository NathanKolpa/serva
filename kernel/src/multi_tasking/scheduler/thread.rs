use core::mem::size_of;

use crate::arch::x86_64::init::GDT;
use crate::arch::x86_64::interrupts::context::{InterruptStackFrame, InterruptedContext};
use crate::arch::x86_64::segmentation::SegmentSelector;
use crate::arch::x86_64::RFlags;
use crate::memory::MemoryMapper;
use crate::util::address::VirtualAddress;

pub struct ThreadStack {
    size: usize,
    top: VirtualAddress,
}

impl ThreadStack {
    pub fn from_slice(slice: &'static mut [u8]) -> Self {
        Self {
            size: slice.len(),
            top: VirtualAddress::from(slice.as_ptr()) + slice.len(),
        }
    }
}

pub enum ThreadState {
    Starting {
        entry_point: VirtualAddress,
        stack: ThreadStack,
    },
    Running {
        state: RunningState,
        context_ptr: InterruptedContext,
    },
}

pub enum RunningState {
    Waiting,
    Blocked { next_blocked: Option<usize> },
    Executing,
}

pub enum ThreadKind {
    Kernel,
    User { memory_map: MemoryMapper },
}

impl ThreadKind {
    fn selectors(&self) -> (SegmentSelector, SegmentSelector) {
        match self {
            Self::Kernel => (GDT.kernel_code, GDT.kernel_data),
            Self::User { .. } => (GDT.user_code, GDT.user_data),
        }
    }
}

pub struct Thread {
    state: ThreadState,
    kind: ThreadKind,
}

impl Thread {
    pub fn new(kind: ThreadKind, stack: ThreadStack, entry_point: VirtualAddress) -> Self {
        Self {
            kind,
            state: ThreadState::Starting { stack, entry_point },
        }
    }

    pub fn is_blocked(&self) -> bool {
        match &self.state {
            ThreadState::Starting { .. } => false,
            ThreadState::Running { state, .. } => match state {
                RunningState::Waiting => false,
                RunningState::Blocked { .. } => true,
                RunningState::Executing => false,
            },
        }
    }

    pub fn unblock(&mut self) -> Option<usize> {
        match &mut self.state {
            ThreadState::Starting { .. } => {}
            ThreadState::Running { state, .. } => match state {
                RunningState::Blocked { next_blocked } => {
                    let next_blocked = *next_blocked;
                    *state = RunningState::Waiting;
                    return next_blocked;
                }
                _ => {}
            },
        }

        None
    }

    pub fn block(&mut self) -> bool {
        match &mut self.state {
            ThreadState::Running { state, .. } => match state {
                RunningState::Waiting | RunningState::Executing => {
                    *state = RunningState::Blocked { next_blocked: None };
                    return true;
                }
                _ => {}
            },
            _ => {}
        }

        false
    }

    pub fn save_context(&mut self, new_context: InterruptedContext) {
        match &mut self.state {
            ThreadState::Running { context_ptr, state } => {
                *context_ptr = new_context;

                match state {
                    RunningState::Waiting => {}
                    RunningState::Blocked { .. } => {}
                    RunningState::Executing => {
                        *state = RunningState::Waiting;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn start(&mut self) -> &InterruptedContext {
        let mut new_stack = None;
        let mut new_entry_point = None;

        match &self.state {
            ThreadState::Starting { stack, entry_point } => {
                new_stack = Some(ThreadStack {
                    top: stack.top,
                    size: stack.size
                });
                new_entry_point = Some(*entry_point);
            },
            _ => {}
        }

        if let (Some(stack_top), Some(entry_point)) = (new_stack, new_entry_point) {
            debug_println!("Init");
            let (code_selector, data_selector) = self.kind.selectors();

            self.state = ThreadState::Running {
                state: RunningState::Executing,
                context_ptr:InterruptedContext::start_new(InterruptStackFrame::new(
                    entry_point,
                    stack_top.top,
                    RFlags::NONE,
                    code_selector,
                    data_selector,
                )),
            };
        }


        match &self.state {
            ThreadState::Running { context_ptr, .. } => context_ptr,
            _ => unreachable!()
        }
    }
}
