use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use crate::arch::x86_64::interrupts::atomic_block;
use crate::multi_tasking::scheduler::{ThreadUnblock, SCHEDULER};
use crate::util::sync::SpinMutex;

const LOCKED: bool = true;
const UNLOCKED: bool = false;

pub struct MutexLockGuard<'a, T> {
    parent: &'a Mutex<T>,
}

impl<'a, T> Deref for MutexLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.parent.data.get() }
    }
}

impl<'a, T> DerefMut for MutexLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.parent.data.get() }
    }
}

impl<'a, T> Drop for MutexLockGuard<'a, T> {
    fn drop(&mut self) {
        self.parent.unlock();
    }
}

pub struct Mutex<T> {
    data: UnsafeCell<T>,
    locked: AtomicBool,
    unblock: SpinMutex<Option<ThreadUnblock>>,
}

unsafe impl<T> Send for Mutex<T> where T: Send + Sync {}
unsafe impl<T> Sync for Mutex<T> where T: Send + Sync {}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            data: UnsafeCell::new(value),
            locked: AtomicBool::new(UNLOCKED),
            unblock: SpinMutex::new(None),
        }
    }

    pub fn lock(&self) -> MutexLockGuard<'_, T> {
        while self
            .locked
            .compare_exchange_weak(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            self.wait_until_unlock();
        }

        MutexLockGuard { parent: self }
    }

    fn wait_until_unlock(&self) {
        atomic_block(|| {
            let mut unblock_lock = self.unblock.lock();

            match unblock_lock.deref_mut() {
                None => *unblock_lock = SCHEDULER.block_next_tick(),
                Some(unblock) => unblock.block_with_after_next_tick()
            }

            drop(unblock_lock);
            SCHEDULER.yield_current();
        });
    }

    fn unlock(&self) {
        self.locked.store(UNLOCKED, Ordering::Release);
        self.unblock_waiting();
    }

    fn unblock_waiting(&self) {
        let mut unblock_lock = self.unblock.lock();


        if let Some(unblock) = unblock_lock.take() {
            *unblock_lock = unblock.unblock_one();

        }
    }
}
