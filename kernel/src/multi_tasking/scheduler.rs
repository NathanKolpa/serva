use core::cell::UnsafeCell;
use core::fmt::{Debug, Formatter};
use core::ops::Deref;
use core::sync::atomic::{AtomicUsize, Ordering};

pub use thread::*;

use crate::arch::x86_64::interrupts::context::InterruptedContext;
use crate::arch::x86_64::interrupts::{atomic_block, int3};
use crate::memory::MemoryMapper;
use crate::util::address::VirtualAddress;
use crate::util::collections::FixedVec;
use crate::util::sync::{PanicOnce, SpinRwLock};

mod thread;
// tegen het advies van Remco in, schrijf ik toch mijn eigen scheduler.

pub struct ThreadUnblock {
    scheduler: &'static Scheduler,
    thread_index: usize,
}

impl Debug for ThreadUnblock {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ThreadUnblock")
            .field("thread_index", &self.thread_index)
            .finish()
    }
}

impl ThreadUnblock {
    pub fn unblock_all(self) {
        unsafe { self.unblock_all_inner() }
    }

    /// # Safety
    /// The caller must ensure that this function is only called once.
    unsafe fn unblock_all_inner(&self) {
        atomic_block(|| {
            let lock = self.scheduler.tasks.read();

            let get_by_index = |index| {
                let thread = lock[index].as_ref().unwrap();
                unsafe { &mut *thread.get() }
            };

            let mut current = get_by_index(self.thread_index);

            while let Some(next) = current.unblock() {
                current = get_by_index(next);
            }

            if let Some(thread) = lock[self.thread_index].as_ref() {
                let thread = unsafe { &mut *thread.get() };
                thread.unblock();
            }
        })
    }

    pub fn unblock_one(self) -> Option<Self> {
        atomic_block(|| {
            let lock = self.scheduler.tasks.read();
            let thread = lock[self.thread_index].as_ref().unwrap();

            let next = unsafe {
                let thread = &mut *thread.get();

                thread.unblock()
            }?;

            Some(Self {
                scheduler: self.scheduler,
                thread_index: next,
            })
        })
    }

    pub fn block_with_after_next_tick(&mut self) {
        if let Some(thread_index) = self
            .scheduler
            .block_next_tick_inner(Some(self.thread_index))
        {
            self.thread_index = thread_index;
        }
    }
}

impl Drop for ThreadUnblock {
    fn drop(&mut self) {
        unsafe { self.unblock_all_inner() }
    }
}

pub struct Scheduler {
    tasks: SpinRwLock<FixedVec<10, Option<UnsafeCell<Thread>>>>,
    current: AtomicUsize,
    default_memory_map: PanicOnce<MemoryMapper>,
    on_exit: PanicOnce<fn() -> !>,
}

impl Scheduler {
    const fn new() -> Self {
        Self {
            current: AtomicUsize::new(0),
            tasks: SpinRwLock::new(FixedVec::new()),
            default_memory_map: PanicOnce::new(),
            on_exit: PanicOnce::new(),
        }
    }

    pub fn initialize(&self, memory_map: MemoryMapper, on_exit: fn() -> !) {
        self.default_memory_map.initialize_with(memory_map);
        self.on_exit.initialize_with(on_exit);
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
        self.yield_current();
        unreachable!()
    }

    pub fn yield_current(&self) {
        int3();
    }

    pub fn block_next_tick(&'static self) -> Option<ThreadUnblock> {
        self.block_next_tick_inner(None)
            .map(|thread_index| ThreadUnblock {
                thread_index,
                scheduler: self,
            })
    }

    fn block_next_tick_inner(&'static self, next: Option<usize>) -> Option<usize> {
        atomic_block(|| {
            let current = {
                let lock = self.tasks.read();
                unsafe { self.current_thread(&lock.as_ref()) }
            };

            current.and_then(|(thread, thread_index)| {
                let taken = thread.block(next);

                if taken {
                    Some(thread_index)
                } else {
                    None
                }
            })
        })
    }

    pub fn get_next_context(&self, ctx: InterruptedContext) -> &InterruptedContext {
        atomic_block(|| {
            let thread_lock = self.tasks.read();
            // its unsafe to call the below function concurrently.
            // however the only concurrency in the kernel is though interrupts.
            // And because this is within an atomic block, this is safe.
            unsafe { self.next_thread_context(&thread_lock.as_ref(), Some(ctx)) }
                .expect("Handle this tho")
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
        ctx: Option<InterruptedContext>,
    ) -> Option<&InterruptedContext> {
        let current_thread = self.current_thread(thread_lock);

        match (ctx, current_thread) {
            (Some(ctx), Some((current_thread, thread_index))) => {
                current_thread.save_context(ctx);
            }
            _ => {}
        }

        self.next_thread(thread_lock).map(|x| x.start())
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
        let start = self.current.fetch_add(1, Ordering::AcqRel);
        let mut next = start + 1;

        loop {
            let next_index = Self::transform_into_index(thread_lock, next)?;
            let next_thread = thread_lock[next_index].as_ref()?;

            let next_thread = &mut *next_thread.get();

            if !next_thread.is_blocked() {
                return Some(next_thread);
            }

            next = self.current.fetch_add(1, Ordering::AcqRel);
        }
    }

    fn exit(&self) -> ! {
        (self.on_exit)();
    }
}

// this is safe because all inner mutable functions are marked as unsafe.
unsafe impl Send for Scheduler {}

unsafe impl Sync for Scheduler {}

pub static SCHEDULER: Scheduler = Scheduler::new();
