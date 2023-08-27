use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};

use crate::arch::x86_64::context::InterruptedContext;
use crate::arch::x86_64::init::GDT;
use crate::arch::x86_64::interrupts::InterruptStackFrame;
use crate::arch::x86_64::segmentation::SegmentSelector;
use crate::arch::x86_64::RFlags;
use crate::memory::MemoryMapper;
use crate::util::address::VirtualAddress;
use crate::util::sync::SpinMutex;

pub struct TaskStack {
    size: usize,
    top: VirtualAddress,
}

impl TaskStack {
    pub fn from_slice(slice: &'static mut [u8]) -> Self {
        Self {
            size: slice.len(),
            top: VirtualAddress::from(slice.as_ptr().wrapping_add(slice.len())),
        }
    }
}

pub enum TaskState {
    Starting {
        entry_point: VirtualAddress,
        stack: TaskStack,
    },
    Running {
        state: RunningState,
        context: MaybeUninit<InterruptedContext>,
    },
}

pub enum RunningState {
    Waiting,
    Blocked,
    Executing,
}

pub enum TaskKind {
    Kernel,
    User { memory_map: MemoryMapper },
}

impl TaskKind {
    fn selectors(&self) -> (SegmentSelector, SegmentSelector) {
        match self {
            Self::Kernel => (GDT.kernel_code, GDT.kernel_data),
            Self::User { .. } => (GDT.user_code, GDT.user_data),
        }
    }
}

pub struct Task {
    id: usize,
    state: SpinMutex<TaskState>,
    kind: TaskKind,
}

impl Task {
    pub fn new(id: usize, kind: TaskKind, stack: TaskStack, entry_point: VirtualAddress) -> Self {
        Self {
            id,
            kind,
            state: SpinMutex::new(TaskState::Starting { stack, entry_point }),
        }
    }

    pub fn save_context(&self, new_context: InterruptedContext) {
        let mut lock = self.state.lock();

        // TODO: set current state as waiting

        match lock.deref_mut() {
            TaskState::Running { context, .. } => {
                context.write(new_context);
            }
            _ => {}
        }
    }

    pub fn run(&self) -> ! {
        let mut lock = self.state.lock();

        let (code_selector, data_selector) = self.kind.selectors();

        match lock.deref_mut() {
            TaskState::Starting { stack, entry_point } => {
                let stack_top = stack.top;
                let entry_point = *entry_point;

                *lock = TaskState::Running {
                    state: RunningState::Executing,
                    context: MaybeUninit::uninit(),
                };

                drop(lock);

                let sf = InterruptStackFrame::new(
                    entry_point,
                    stack_top,
                    RFlags::INTERRUPTS_ENABLED,
                    code_selector,
                    data_selector,
                );

                unsafe {
                    sf.iretq()
                }
            }
            TaskState::Running { context, .. } => unsafe {
                let ctx = context.assume_init_ref().clone();
                drop(lock);
                ctx.restore()
            },
        }
    }
}
