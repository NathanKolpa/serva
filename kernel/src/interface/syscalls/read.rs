use crate::arch::x86_64::syscalls::SyscallArgs;
use crate::interface::syscalls::SyscallResult;
use crate::service::ServiceRef;

pub fn read_syscall(_args: &SyscallArgs, _current_service: ServiceRef) -> SyscallResult {
    Ok(0)
}
