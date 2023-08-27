use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub use task::*;
use crate::arch::x86_64::context::InterruptedContext;

use crate::memory::MemoryMapper;
use crate::util::address::VirtualAddress;
use crate::util::collections::FixedVec;
use crate::util::sync::{SpinMutex, SpinRwLock};
use crate::util::InitializeGuard;

mod task;
// tegen het advies van Remco in, schrijf ik toch mijn eigen scheduler.

pub struct Scheduler {
    tasks: SpinRwLock<FixedVec<10, Option<Task>>>,
    current: SpinMutex<Option<usize>>,
    default_memory_map: UnsafeCell<MaybeUninit<MemoryMapper>>,
    on_exit: UnsafeCell<MaybeUninit<fn() -> !>>,
    initialized: InitializeGuard,
}

impl Scheduler {
    const fn new() -> Self {
        Self {
            current: SpinMutex::new(None),
            tasks: SpinRwLock::new(FixedVec::new()),
            default_memory_map: UnsafeCell::new(MaybeUninit::uninit()),
            on_exit: UnsafeCell::new(MaybeUninit::uninit()),
            initialized: InitializeGuard::new(),
        }
    }

    pub fn initialize(&self, memory_map: MemoryMapper, on_exit: fn() -> !) {
        self.initialized.guard();

        unsafe {
            *self.default_memory_map.get() = MaybeUninit::new(memory_map);
            *self.on_exit.get() = MaybeUninit::new(on_exit);
        }
    }

    pub fn add_kernel_task(&self, stack: TaskStack, main: fn() -> !) {
        self.add_task(
            TaskKind::Kernel,
            stack,
            VirtualAddress::from(main as *const fn()),
        )
    }

    fn add_task(&self, kind: TaskKind, stack: TaskStack, entry_point: VirtualAddress) {
        let mut task_lock = self.tasks.write();

        let id;
        let slot;

        let search_result = task_lock.iter_mut().enumerate().find(|(_, x)| x.is_none());
        match search_result {
            None => {
                if task_lock.is_full() {
                    panic!("Exceeded maximum amount of tasks");
                }

                id = task_lock.len();
                task_lock.push(None);
                slot = &mut task_lock[id];
            }
            Some(search_result) => {
                id = search_result.0;
                slot = search_result.1;
            }
        }

        *slot = Some(Task::new(id, kind, stack, entry_point));
    }

    pub fn start(&self) -> ! {
        self.initialized.assert_initialized();
        self.next_task()
    }

    pub unsafe fn next_tick(&self, ctx: InterruptedContext) -> ! {
        self.save_current(ctx);
        self.next_task()
    }

    fn save_current(&self, ctx: InterruptedContext) {
        let Some(current) = *self.current.lock() else {
            return;
        };

        let tasks = self.tasks.read();
        tasks[current].as_ref().unwrap().save_context(ctx);
    }

    fn next_task(&self) -> ! {
        let tasks = self.tasks.read();

        let next = {
            let mut current = self.current.lock();
            let start_offset = current.unwrap_or(0);
            let diff = current.map(|_| 1).unwrap_or(0);
            let mut next = None;

            for i in (0..tasks.len()).map(|i| (i + start_offset + diff) % tasks.len()) {
                let task = &tasks[i];

                if let Some(_task) = task {// TODO: check task conds
                    next = Some(i);
                    break;
                }
            }

            *current = next;
            next
        };

        match next {
            None => self.exit(),
            Some(next) => {
                let task = tasks[next].as_ref().unwrap();

                task.run()
            }
        }
    }

    fn exit(&self) -> ! {
        let exit_fn = unsafe { (*self.on_exit.get()).assume_init() };
        exit_fn()
    }
}

unsafe impl Send for Scheduler {}
unsafe impl Sync for Scheduler {}

pub static SCHEDULER: Scheduler = Scheduler::new();
