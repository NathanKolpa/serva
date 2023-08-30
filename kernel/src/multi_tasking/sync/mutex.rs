use crate::multi_tasking::scheduler::{ThreadUnblock, SCHEDULER};
use crate::util::sync::SpinMutex;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

const LOCKED: bool = true;
const UNLOCKED: bool = false;

const UNBLOCK_SET: u8 = 0;
const UNBLOCK_UNSET: u8 = 1;

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
    unblock_state: AtomicU8,
    unblock: UnsafeCell<Option<ThreadUnblock>>,
}

unsafe impl<T> Send for Mutex<T> where T: Send + Sync {}
unsafe impl<T> Sync for Mutex<T> where T: Send + Sync {}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            data: UnsafeCell::new(value),
            locked: AtomicBool::new(UNLOCKED),
            unblock_state: AtomicU8::new(UNBLOCK_UNSET),
            unblock: UnsafeCell::new(None),
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
        if self
            .unblock_state
            .compare_exchange_weak(
                UNBLOCK_UNSET,
                UNBLOCK_SET,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_ok()
        {
            debug_println!("Wait utill unlock");

            unsafe {
                let unblock = (&mut *self.unblock.get());
                SCHEDULER.block_and_yield_current(unblock);
            }

            debug_println!("Unlocked!");
        }
    }

    fn unlock(&self) {
        self.locked.store(UNLOCKED, Ordering::Release);
        self.unblock_waiting();
    }

    fn unblock_waiting(&self) {
        if self.unblock_state.load(Ordering::Relaxed) == UNBLOCK_SET {
            unsafe {
                let current_unblock = (&mut *self.unblock.get());

                debug_println!("Unlock {current_unblock:?}");

                if let Some(unblock) = current_unblock.take() {
                    *current_unblock = unblock.unblock_one();
                }
            }

            self.unblock_state.store(UNBLOCK_UNSET, Ordering::Release);
        }
    }
}
