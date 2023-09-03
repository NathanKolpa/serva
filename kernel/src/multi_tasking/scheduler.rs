pub use stack::*;
pub use thread::*;

use crate::arch::x86_64::interrupts::context::InterruptedContext;
use crate::arch::x86_64::interrupts::int3;
use crate::util::collections::FixedVec;
use crate::util::sync::SpinMutex;

mod stack;
mod thread;

pub struct Scheduler {
    current: SpinMutex<Option<ThreadId>>,
    tasks: SpinMutex<FixedVec<10, Thread>>,
}

impl Scheduler {
    const fn new() -> Self {
        Self {
            current: SpinMutex::new(None),
            tasks: SpinMutex::new(FixedVec::new()),
        }
    }

    pub fn add_thread(&self, thread: Thread) {
        let mut lock = self.tasks.lock();
        lock.push(thread);
    }

    pub fn tick(&self, ctx: InterruptedContext) -> *const InterruptedContext {
        self.save_and_set_waiting(ctx);
        self.get_next()
    }

    pub fn yield_current(&self) -> ! {
        int3();
        unreachable!()
    }

    fn save_and_set_waiting(&self, ctx: InterruptedContext) {
        let current_lock = self.current.lock();

        if let Some(current) = *current_lock {
            let mut tasks_lock = self.tasks.lock();

            let current_task = &mut tasks_lock[current];
            current_task.save(ctx);
            current_task.set_state(ThreadState::Waiting);
        }
    }

    fn get_next(&self) -> *const InterruptedContext {
        let mut current_lock = self.current.lock();
        let mut tasks_lock = self.tasks.lock();

        let task_count = tasks_lock.len();

        let current = current_lock.unwrap_or(0);

        let next_thread_id = (0..task_count)
            .map(|i| (i + current) % task_count)
            .find_map(|id| {
                let task = &tasks_lock[id];

                if !task.can_run() {
                    return None;
                }

                Some(id)
            });

        let Some(next_thread_id) = next_thread_id else {
            todo!("Handle exit condition")
        };

        *current_lock = Some(next_thread_id);

        let next_thread = &mut tasks_lock[next_thread_id];
        next_thread.set_state(ThreadState::Running);
        next_thread.context_ptr()
    }
}

pub static SCHEDULER: Scheduler = Scheduler::new();
