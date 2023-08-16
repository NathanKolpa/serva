//! Concurrency primitives

pub use expected::Expected;
pub use spin::{SpinMutex, SpinRwLock};

mod expected;
mod spin;
