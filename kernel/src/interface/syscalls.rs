use syscall::{encode_syscall, SyscallError, SyscallResult};

use crate::arch::x86_64::interrupts::atomic_block;
use crate::arch::x86_64::syscalls::SyscallArgs;
use crate::multi_tasking::scheduler::SCHEDULER;
use crate::service::ServiceRef;

mod accept;
mod connect;
mod disconnect;
mod hello;
mod read;
mod request;
mod write;

pub type SyscallHandler = fn(&SyscallArgs, ServiceRef) -> SyscallResult;

static USER_SYSCALL_TABLE: [SyscallHandler; 6] = [
    hello::hello_syscall,
    connect::connect_syscall,
    request::request_syscall,
    write::write_syscall,
    read::read_syscall,
    accept::accept_syscall,
];

static KERNEL_SYSCALL_TABLE: [SyscallHandler; 0] = [];

const KERNEL_CALLS_START: usize = 1024;

pub fn handle_kernel_syscall(args: &SyscallArgs) -> SyscallResult {
    let mut call_index = args.syscall as usize;

    let table = if call_index >= KERNEL_CALLS_START {
        call_index -= KERNEL_CALLS_START;
        KERNEL_SYSCALL_TABLE.as_slice()
    } else {
        USER_SYSCALL_TABLE.as_slice()
    };

    if call_index > table.len() {
        return Err(SyscallError::UnknownSyscall);
    }

    let current_service = atomic_block(|| {
        SCHEDULER
            .current_service()
            .expect("syscalls should only be called from services")
    });

    (table[call_index])(&args, current_service)
}
pub fn handle_user_syscall(args: &SyscallArgs) -> SyscallResult {
    let call_index = args.syscall as usize;

    if call_index >= KERNEL_CALLS_START {
        return Err(SyscallError::OperationNotPermitted);
    }

    handle_kernel_syscall(args)
}

pub fn handle_user_syscall_raw(args: SyscallArgs) -> u64 {
    let result = handle_user_syscall(&args);
    encode_syscall(result)
}
