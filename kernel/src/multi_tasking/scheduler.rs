use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ops::Deref;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::arch::x86_64::interrupts::context::InterruptedContext;
pub use task::*;

use crate::memory::MemoryMapper;
use crate::util::address::VirtualAddress;
use crate::util::collections::FixedVec;
use crate::util::sync::{SpinMutex, SpinRwLock};
use crate::util::InitializeGuard;

mod task;
// tegen het advies van Remco in, schrijf ik toch mijn eigen scheduler.

pub struct Scheduler {
    tasks: SpinRwLock<FixedVec<10, Option<Thread>>>,
    current: AtomicUsize,
    default_memory_map: UnsafeCell<MaybeUninit<MemoryMapper>>,
    on_exit: UnsafeCell<MaybeUninit<fn() -> !>>,
    initialized: InitializeGuard,
}

impl Scheduler {
    const fn new() -> Self {
        Self {
            current: AtomicUsize::new(0),
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

    pub fn new_kernel_thread(&self, stack: ThreadStack, main: fn() -> !) {
        self.new_thread(
            ThreadKind::Kernel,
            stack,
            VirtualAddress::from(main as *const fn()),
        )
    }

    fn new_thread(&self, kind: ThreadKind, stack: ThreadStack, entry_point: VirtualAddress) {
        let mut task_lock = self.tasks.write();

        let slot;

        let search_result = task_lock.iter_mut().find(|x| x.is_none());
        match search_result {
            None => {
                if task_lock.is_full() {
                    panic!("Exceeded maximum amount of tasks");
                }

                let id = task_lock.len();
                task_lock.push(None);
                slot = &mut task_lock[id];
            }
            Some(search_result) => {
                slot = search_result;
            }
        }

        *slot = Some(Thread::new(kind, stack, entry_point));
    }

    pub fn start(&self) -> ! {
        self.initialized.assert_initialized();

        let thread_lock = self.tasks.read();

        let ctx = self.next_thread_context(&thread_lock.as_ref(), None);

        let Some(ctx) = ctx else {
            self.exit();
        };

        unsafe { (&*ctx).interrupt_stack_frame.iretq() }
    }

    pub fn next_context(
        &self,
        ctx: *const InterruptedContext,
    ) -> Option<*const InterruptedContext> {
        self.initialized.assert_initialized();
        let thread_lock = self.tasks.read();
        self.next_thread_context(&thread_lock.as_ref(), Some(ctx))
    }

    fn save_current<L: Deref<Target = [Option<Thread>]>>(
        &self,
        thread_lock: &L,
        current: usize,
        ctx: *const InterruptedContext,
    ) {
        thread_lock[current].as_ref().unwrap().save_context(ctx);
    }

    fn transform_into_index<L: Deref<Target = [Option<Thread>]>>(
        thread_lock: &L,
        current: usize,
    ) -> Option<usize> {
        if thread_lock.is_empty() {
            return None;
        }

        Some(current % thread_lock.len())
    }

    fn next_thread_context<L: Deref<Target = [Option<Thread>]>>(
        &self,
        thread_lock: &L,
        ctx: Option<*const InterruptedContext>,
    ) -> Option<*const InterruptedContext> {
        let current = self.current.fetch_add(1, Ordering::SeqCst);
        let current_index = Self::transform_into_index(thread_lock, current);

        let current_thread = current_index
            .as_ref()
            .and_then(|current| thread_lock[*current].as_ref());

        match (ctx, current_thread) {
            (Some(ctx), Some(current_thread)) => {
                current_thread.save_context(ctx);
            }
            _ => {}
        }

        Self::transform_into_index(thread_lock.clone(), current + 1)
            .and_then(|index| thread_lock[index].as_ref())
            .map(|thread| thread.run_next())
    }

    fn exit(&self) -> ! {
        let exit_fn = unsafe { (*self.on_exit.get()).assume_init() };
        exit_fn()
    }
}

unsafe impl Send for Scheduler {}

unsafe impl Sync for Scheduler {}

pub static SCHEDULER: Scheduler = Scheduler::new();
