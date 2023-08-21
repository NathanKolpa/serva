//! Simple concurrency primitives for limited environments.
//!
//! ## Note:
//!
//! Spin-locks are very resource inefficient, please use the [`crate::tasks::sync`] module where possible.

pub use once::SpinOnce;
pub use spin::{SpinMutex, SpinRwLock};

mod once;
mod spin;
