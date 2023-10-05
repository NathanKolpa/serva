use crate::interface::syscalls::{SyscallError, SyscallResult};
use crate::service::{Id, ReadError, ServiceRef};
use essentials::address::VirtualAddress;
use x86_64::interrupts::atomic_block;
use x86_64::syscalls::SyscallArgs;

pub fn read_syscall(args: &SyscallArgs, current_service: ServiceRef) -> SyscallResult {
    let connection_id = args.arg0 as Id;
    let buffer_len = args.arg1 as usize;
    let buffer_ptr = args.arg2;

    let Some(read_buffer) =
        atomic_block(|| current_service.deref_incoming_pointer(VirtualAddress::from(buffer_ptr)))
    else {
        return Err(SyscallError::InvalidPointerMappings);
    };

    if buffer_len > read_buffer.len() {
        return Err(SyscallError::InvalidPointerMappings);
    }

    let target_buffer = &mut read_buffer[0..buffer_len];
    let mut start = 0;

    atomic_block(|| loop {
        let result = current_service.read(connection_id, target_buffer, start);

        match result {
            Err(err) => {
                return match err {
                    ReadError::InvalidConnection => Err(SyscallError::ResourceNotFound),
                    ReadError::RequestClosed => Ok(0),
                }
            }
            Ok(read) => {
                start += read;

                if start > 0 || target_buffer.is_empty() {
                    return Ok(start as u64);
                }

                // because the buffer could not be written in its entirety, it must be full.
                // Therefore wait until the other side reads from the buffer.
                current_service.block_until_read_available(connection_id)
            }
        }
    })
}
