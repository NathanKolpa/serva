use crate::arch::x86_64::interrupts::atomic_block;
use crate::arch::x86_64::syscalls::SyscallArgs;
use crate::service::{Id, ServiceRef};
use core::mem::size_of;
use syscall::SyscallResult;

const NEW_CONNECTION_FLAG: u64 = 1 << (size_of::<Id>() * 2 * 8);

pub fn accept_syscall(_args: &SyscallArgs, current_service: ServiceRef) -> SyscallResult {
    atomic_block(|| loop {
        let next_connection = current_service.accept_next_connection_request();

        if let Some((id, endpoint)) = next_connection {
            let mut result = 0;
            result |= id as u64;
            result |= (endpoint as u64) << (size_of::<Id>() * 8);
            return Ok(result | NEW_CONNECTION_FLAG);
        }

        current_service.block_until_next_request();
    })
}
