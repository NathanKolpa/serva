use core::sync::atomic::{AtomicBool, Ordering};

pub struct InitializeGuard {
    is_initialized: AtomicBool,
}

impl InitializeGuard {
    pub const fn new() -> Self {
        Self {
            is_initialized: AtomicBool::new(false),
        }
    }

    pub fn guard(&self) {
        if self
            .is_initialized
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            panic!("Already initialized");
        }
    }

    pub fn assert_initialized(&self) {
        if !self.is_initialized.load(Ordering::Relaxed) {
            panic!("Not initialized");
        }
    }
}
