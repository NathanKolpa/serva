mod connect;
mod disconnect;
mod error;
mod hello;
mod request;
mod write;

use crate::arch::x86_64::syscalls::SyscallArgs;

use crate::service::Privilege;
use error::encode_result;
pub use error::{SyscallError, SyscallResult};

pub type SyscallHandler = fn(&SyscallArgs) -> SyscallResult;

const EXPECT_CURRENT_SERVICE: &str = "syscalls can only be called from services";

static SYSCALL_TABLE: [SyscallHandler; 4] = [
    hello::hello_syscall,
    connect::connect_syscall,
    request::request_syscall,
    write::request_syscall
];

const USER_CALLS_START: usize = 0;

pub fn handle_kernel_syscall(args: &SyscallArgs) -> SyscallResult {
    let call_index = args.syscall as usize;

    if call_index > SYSCALL_TABLE.len() {
        return Err(SyscallError::UnknownSyscall);
    }

    (SYSCALL_TABLE[call_index])(&args)
}
pub fn handle_user_syscall(args: &SyscallArgs) -> SyscallResult {
    let call_index = args.syscall as usize;

    if call_index < USER_CALLS_START {
        return Err(SyscallError::OperationNotPermitted);
    }

    handle_kernel_syscall(args)
}

pub fn handle_user_syscall_raw(args: SyscallArgs) -> u64 {
    let result = handle_user_syscall(&args);
    encode_result(result)
}
