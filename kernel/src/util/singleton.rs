use core::ops::Deref;

use crate::util::sync::{SpinOnce};

pub struct Singleton<T> {
    value: SpinOnce<T>,
    initializer: fn() -> T,
}

impl<T> Singleton<T> {
    pub const fn new(initializer: fn() -> T) -> Self {
        Self {
            initializer,
            value: SpinOnce::new()
        }
    }
}

impl<T> Deref for Singleton<T>  {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.call_once(self.initializer)
    }
}