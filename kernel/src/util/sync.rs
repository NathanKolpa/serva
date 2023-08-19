//! Concurrency primitives

pub use once::SpinOnce;
pub use spin::{SpinMutex, SpinRwLock};

mod once;
mod spin;
