use crate::arch::x86_64::syscalls::SyscallArgs;
use crate::interface::syscalls::SyscallResult;

pub fn connect_syscall(args: &SyscallArgs) -> SyscallResult {
    // debug_println!("Hello syscall! Args: {args:?}");
    Ok(0)
}
