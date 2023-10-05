//! Simple concurrency primitives for limited environments.
//!
//! ## Note:
//!
//! Spin-locks are very resource inefficient, please use the [`crate::multi_tasking::sync`] module where possible.

pub use expected::Expected;
pub use panic_once::PanicOnce;
pub use singleton::Singleton;
pub use spin::{SpinMutex, SpinRwLock};
pub use spin_once::SpinOnce;

mod expected;
mod panic_once;
mod singleton;
mod spin;
mod spin_once;
