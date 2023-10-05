use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicU8, Ordering};

const INITIALIZED: u8 = 1;

const INITIALIZING: u8 = 2;
const NOT_INITIALIZED: u8 = 3;

pub struct PanicOnce<T> {
    initialized: AtomicU8,
    data: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T> Send for PanicOnce<T> where T: Send + Sync {}
unsafe impl<T> Sync for PanicOnce<T> where T: Send + Sync {}

impl<T> PanicOnce<T> {
    pub const fn new() -> Self {
        Self {
            data: UnsafeCell::new(MaybeUninit::uninit()),
            initialized: AtomicU8::new(NOT_INITIALIZED),
        }
    }

    pub fn initialize_with(&self, value: T) {
        if self
            .initialized
            .compare_exchange_weak(
                NOT_INITIALIZED,
                INITIALIZING,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_err()
        {
            panic!("Already initialized");
        }

        unsafe {
            (&mut *self.data.get()).write(value);
        }

        self.initialized.store(INITIALIZED, Ordering::Release);
    }

    fn guard(&self) {
        if self.is_initialized() {
            panic!("Not initialized");
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Relaxed) != INITIALIZED
    }
}

impl<T> Deref for PanicOnce<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard();

        unsafe { (&*self.data.get()).assume_init_ref() }
    }
}

impl<T> DerefMut for PanicOnce<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard();

        unsafe { (&mut *self.data.get()).assume_init_mut() }
    }
}

impl<T> Drop for PanicOnce<T> {
    fn drop(&mut self) {
        if self.is_initialized() {
            unsafe { (&mut *self.data.get()).assume_init_drop() }
        }
    }
}
