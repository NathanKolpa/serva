use crate::interface::syscalls::SyscallResult;
use crate::service::ServiceRef;
use x86_64::syscalls::SyscallArgs;

/// A syscall to help services implement their syscall interface.
pub fn hello_syscall(args: &SyscallArgs, current_service: ServiceRef) -> SyscallResult {
    debug_println!("Hello service #{}! Args: {args:?}", current_service.id());
    Ok(0)
}
