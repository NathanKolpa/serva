use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ops::Deref;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::arch::x86_64::interrupts::atomic_block;
use crate::arch::x86_64::interrupts::context::InterruptedContext;
pub use thread::*;

use crate::memory::MemoryMapper;
use crate::util::address::VirtualAddress;
use crate::util::collections::FixedVec;
use crate::util::sync::{SpinMutex, SpinRwLock};
use crate::util::InitializeGuard;

mod thread;
// tegen het advies van Remco in, schrijf ik toch mijn eigen scheduler.

pub struct ThreadUnblock<'a> {
    scheduler: &'a Scheduler,
    thread_index: usize,
}

impl Drop for ThreadUnblock<'_> {
    fn drop(&mut self) {
        let lock = self.scheduler.tasks.read();
        if let Some(thread) = lock[self.thread_index].as_ref() {
            let thread = unsafe { &mut *thread.get() };
            thread.unblock();
        }
    }
}

pub struct Scheduler {
    tasks: SpinRwLock<FixedVec<10, Option<UnsafeCell<Thread>>>>,
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

        *slot = Some(UnsafeCell::new(Thread::new(kind, stack, entry_point)));
    }

    pub unsafe fn start(&self) -> ! {
        self.initialized.assert_initialized();

        let thread_lock = self.tasks.read();

        let (ctx, _) = self.next_thread_context(&thread_lock.as_ref(), None);

        let Some(ctx) = ctx else {
            unsafe { self.exit() };
        };

        (&*ctx).interrupt_stack_frame.iretq()
    }

    pub fn block_current_and_get_next(
        &self,
        ctx: *const InterruptedContext,
    ) -> (Option<*const InterruptedContext>, ThreadUnblock) {
        let (new_ctx, unblock) = self.next_context_inner(ctx, false);

        (new_ctx, unblock.unwrap())
    }

    pub fn get_next_context(
        &self,
        ctx: *const InterruptedContext,
    ) -> Option<*const InterruptedContext> {
        self.next_context_inner(ctx, false).0
    }

    fn next_context_inner(
        &self,
        ctx: *const InterruptedContext,
        blocked: bool,
    ) -> (Option<*const InterruptedContext>, Option<ThreadUnblock>) {
        self.initialized.assert_initialized();

        atomic_block(|| {
            let thread_lock = self.tasks.read();
            // its unsafe to call the below function concurrently.
            // however the only concurrency in the kernel is though interrupts.
            // And because this is within an atomic block, this is safe.
            unsafe { self.next_thread_context(&thread_lock.as_ref(), Some((ctx, blocked))) }
        })
    }

    fn transform_into_index<L: Deref<Target = [Option<UnsafeCell<Thread>>]>>(
        thread_lock: &L,
        current: usize,
    ) -> Option<usize> {
        if thread_lock.is_empty() {
            return None;
        }

        Some(current % thread_lock.len())
    }

    unsafe fn next_thread_context<L: Deref<Target = [Option<UnsafeCell<Thread>>]>>(
        &self,
        thread_lock: &L,
        ctx: Option<(*const InterruptedContext, bool)>,
    ) -> (Option<*const InterruptedContext>, Option<ThreadUnblock>) {
        let current_thread = self.current_thread(thread_lock);
        let mut unblock = None;

        match (ctx, current_thread) {
            (Some((ctx, blocked)), Some((current_thread, thread_index))) => {
                current_thread.save_context(ctx, blocked);

                if blocked {
                    unblock = Some(ThreadUnblock {
                        thread_index,
                        scheduler: self,
                    });
                }
            }
            _ => {}
        }

        loop {
            let new_ctx = self.next_thread(thread_lock).map(|x| x.start());

            if new_ctx.is_some() {
                return (new_ctx, unblock);
            }
        }
    }

    unsafe fn current_thread<L: Deref<Target = [Option<UnsafeCell<Thread>>]>>(
        &self,
        thread_lock: &L,
    ) -> Option<(&mut Thread, usize)> {
        let current = self.current.load(Ordering::Relaxed);
        let current_index = Self::transform_into_index(thread_lock, current)?;
        let current_thread = &thread_lock[current_index].as_ref()?;

        Some((&mut *current_thread.get(), current_index))
    }

    unsafe fn next_thread<L: Deref<Target = [Option<UnsafeCell<Thread>>]>>(
        &self,
        thread_lock: &L,
    ) -> Option<&mut Thread> {
        let start = self.current.fetch_add(1, Ordering::SeqCst);
        let mut next = start + 1;

        loop {
            let next_index = Self::transform_into_index(thread_lock, next)?;
            let next_thread = thread_lock[next_index].as_ref()?;

            let next_thread = &mut *next_thread.get();

            if !next_thread.is_blocked() {
                return Some(next_thread);
            }

            next = self.current.fetch_add(1, Ordering::SeqCst);
        }
    }

    unsafe fn exit(&self) -> ! {
        let exit_fn = (*self.on_exit.get()).assume_init();
        exit_fn()
    }
}

// this is safe because all inner mutable functions are marked as unsafe.
unsafe impl Send for Scheduler {}

unsafe impl Sync for Scheduler {}

pub static SCHEDULER: Scheduler = Scheduler::new();
