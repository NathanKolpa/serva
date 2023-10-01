use crate::arch::x86_64::interrupts::atomic_block;
use crate::arch::x86_64::syscalls::SyscallArgs;
use crate::interface::syscalls::{SyscallError, SyscallResult};
use crate::service::{Id, ReadError, ServiceRef};
use crate::util::address::VirtualAddress;

fn map_read_error_to_syscall_error(err: ReadError) -> SyscallError {
    match err {
        ReadError::InvalidConnection => SyscallError::ResourceNotFound,
        ReadError::RequestClosed => SyscallError::RequestClosed,
    }
}

pub fn read_syscall(args: &SyscallArgs, current_service: ServiceRef) -> SyscallResult {
    let connection_id = args.arg0 as Id;
    let buffer_size = args.arg2 as usize;

    let Some(read_buffer) =
        atomic_block(|| current_service.deref_incoming_pointer(VirtualAddress::from(args.arg1)))
        else {
            return Err(SyscallError::InvalidPointerMappings);
        };

    let target_buffer = &mut read_buffer[0..buffer_size];
    let mut start = 0;

    atomic_block(|| loop {
        let result = current_service.read(connection_id, target_buffer, start);

        match result {
            Err(err) => return Err(map_read_error_to_syscall_error(err)),
            Ok(read) => {
                start += read;

                if start >= target_buffer.len() {
                    return Ok(start as u64);
                }

                debug_println!("Wating");

                // because the buffer could not be written in its entirety, it must be full.
                // Therefore wait until the other side reads from the buffer.
                current_service.block_until_read_available(connection_id)
            }
        }
    })
}
