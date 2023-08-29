use core::mem::{size_of, MaybeUninit};
use core::ops::{Deref, DerefMut};

use crate::arch::x86_64::init::GDT;
use crate::arch::x86_64::interrupts::context::{InterruptStackFrame, InterruptedContext};
use crate::arch::x86_64::segmentation::SegmentSelector;
use crate::arch::x86_64::RFlags;
use crate::memory::MemoryMapper;
use crate::util::address::VirtualAddress;
use crate::util::sync::SpinMutex;

pub struct ThreadStack {
    size: usize,
    top: VirtualAddress,
}

impl ThreadStack {
    pub fn from_slice(slice: &'static mut [u8]) -> Self {
        Self {
            size: slice.len(),
            top: VirtualAddress::from(slice.as_ptr().wrapping_add(slice.len())),
        }
    }

    pub fn ctx_mut(&mut self) -> *mut InterruptedContext {
        const CTX_SIZE: usize = size_of::<InterruptedContext>();

        assert!(CTX_SIZE < self.size);
        (self.top.as_usize() - CTX_SIZE) as *mut InterruptedContext
    }
}

pub enum ThreadState {
    Starting {
        entry_point: VirtualAddress,
        stack: ThreadStack,
    },
    Running {
        state: RunningState,
        context_ptr: *const InterruptedContext,
    },
}

pub enum RunningState {
    Waiting,
    Blocked,
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
                RunningState::Blocked => true,
                RunningState::Executing => false,
            },
        }
    }

    pub fn save_context(&mut self, new_context: *const InterruptedContext) {
        match &mut self.state {
            ThreadState::Running { context_ptr, state } => {
                *context_ptr = new_context;
                *state = RunningState::Waiting;
            }
            _ => {}
        }
    }

    pub fn start(&mut self) -> *const InterruptedContext {
        match &mut self.state {
            ThreadState::Starting { stack, entry_point } => {
                let (code_selector, data_selector) = self.kind.selectors();

                let stack_top = stack.top;
                let entry_point = *entry_point;

                let ctx = stack.ctx_mut();

                unsafe {
                    *ctx = InterruptedContext::start_new(InterruptStackFrame::new(
                        entry_point,
                        stack_top,
                        RFlags::INTERRUPTS_ENABLED,
                        code_selector,
                        data_selector,
                    ));
                }

                self.state = ThreadState::Running {
                    state: RunningState::Executing,
                    context_ptr: ctx,
                };

                ctx
            }
            ThreadState::Running { context_ptr, .. } => *context_ptr,
        }
    }
}
