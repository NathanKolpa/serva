//! Typed (and generally unsafe) wrappers around syscalls.

mod error;

pub use error::*;

/// # Safety
///
/// This function is unsafe because arguments can be interpreted as pointers.
/// The caller must ensure that the rust borrow checker rule's are respected on order to guarantee safety.
pub unsafe fn syscall(syscall: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64) {

}
