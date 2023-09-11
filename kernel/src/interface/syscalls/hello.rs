use crate::arch::x86_64::syscalls::SyscallArgs;
use crate::interface::syscalls::SyscallResult;

/// A syscall to help services implement their syscall interface.
pub fn hello_syscall(args: &SyscallArgs) -> SyscallResult {
    debug_println!("Hello syscall! Args: {args:?}");
    Ok(0)
}
