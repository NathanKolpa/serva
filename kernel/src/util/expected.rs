use core::ops::{Deref, DerefMut};

pub struct Expected<T> {
    value: Option<T>,
}

impl<T> Expected<T> {
    pub const fn new() -> Self {
        Self { value: None }
    }

    pub fn set(&mut self, value: T) {
        self.value = Some(value);
    }
}

const EXPECT_MSG: &str = "Expected value not initialized";

impl<T> Deref for Expected<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref().expect(EXPECT_MSG)
    }
}

impl<T> DerefMut for Expected<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.as_mut().expect(EXPECT_MSG)
    }
}
