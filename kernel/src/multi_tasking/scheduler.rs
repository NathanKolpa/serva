pub use stack::*;
pub use thread::*;

use crate::arch::x86_64::interrupts::context::InterruptedContext;
use crate::arch::x86_64::interrupts::int3;
use crate::service::{ServiceRef, SERVICE_TABLE};
use crate::util::collections::FixedVec;
use crate::util::sync::SpinMutex;

mod stack;
mod thread;

pub struct ThreadBlocker {
    thread_id: ThreadId,
    last_thread_id: ThreadId,
    scheduler: &'static Scheduler,
    should_drop: bool,
}

impl ThreadBlocker {
    pub fn unblock_one(mut self) -> Option<ThreadBlocker> {
        self.should_drop = false;

        let mut tasks = self.scheduler.tasks.lock();
        tasks[self.thread_id].unblock().map(|next| ThreadBlocker {
            thread_id: next,
            scheduler: self.scheduler,
            last_thread_id: self.last_thread_id,
            should_drop: true
        })
    }

    pub fn block_current(&mut self) {
        let next_block = self.scheduler.block_current();
        let mut tasks = self.scheduler.tasks.lock();
        tasks[self.last_thread_id].set_next_block(next_block.thread_id);
        self.last_thread_id = next_block.thread_id;
    }
}

impl Drop for ThreadBlocker {
    fn drop(&mut self) {
        if !self.should_drop {
            return;
        }

        let mut current = ThreadBlocker {
            scheduler: self.scheduler,
            thread_id: self.thread_id,
            last_thread_id: self.last_thread_id,
            should_drop: false,
        };

        while let Some(next) = current.unblock_one() {
            current = next;
        }
    }
}

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

    pub fn current_service(&self) -> Option<ServiceRef> {
        let current_lock = self.current.lock();
        let tasks_lock = self.tasks.lock();

        current_lock
            .and_then(|thread| tasks_lock[thread].service_id())
            .map(|service_id| ServiceRef::new(&SERVICE_TABLE, service_id))
    }

    pub fn add_thread(&self, thread: Thread) {
        let mut lock = self.tasks.lock();
        lock.push(thread);
    }

    pub fn block_current(&'static self) -> ThreadBlocker {
        let current = self
            .current
            .lock()
            .expect("cannot block threads when the scheduler is not yet started");
        let mut tasks_lock = self.tasks.lock();

        tasks_lock[current].block();
        ThreadBlocker {
            scheduler: self,
            thread_id: current,
            last_thread_id: current,
            should_drop: true
        }
    }

    pub fn tick(
        &self,
        ctx: InterruptedContext,
    ) -> (*const InterruptedContext, Option<ServiceRef<'static>>) {
        self.save_and_set_waiting(ctx);
        self.get_next()
    }

    pub fn yield_current(&self) {
        int3();
    }

    fn save_and_set_waiting(&self, ctx: InterruptedContext) {
        let current_lock = self.current.lock();

        if let Some(current) = *current_lock {
            let mut tasks_lock = self.tasks.lock();

            let current_task = &mut tasks_lock[current];
            current_task.save(ctx);
            current_task.finish_tick();
        }
    }

    fn get_next(&self) -> (*const InterruptedContext, Option<ServiceRef<'static>>) {
        let mut current_lock = self.current.lock();
        let mut tasks_lock = self.tasks.lock();

        let task_count = tasks_lock.len();

        let start = current_lock.map(|current| current + 1).unwrap_or(0);

        let next_thread_id = (0..task_count)
            .map(|i| (i + start) % task_count)
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
        next_thread.start_tick();
        (
            next_thread.context_ptr(),
            next_thread
                .service_id()
                .map(|id| ServiceRef::new(&SERVICE_TABLE, id)),
        )
    }
}

pub static SCHEDULER: Scheduler = Scheduler::new();
