//! Simple concurrency primitives for limited environments.
//!
//! ## Note:
//!
//! Spin-locks are very resource inefficient, please use the [`crate::multi_tasking::sync`] module where possible.

pub use spin_once::SpinOnce;
pub use spin::{SpinMutex, SpinRwLock};
pub use panic_once::PanicOnce;

mod spin_once;
mod spin;
mod panic_once;
