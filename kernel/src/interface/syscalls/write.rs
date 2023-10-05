use crate::arch::x86_64::interrupts::atomic_block;
use crate::arch::x86_64::syscalls::SyscallArgs;
use crate::interface::syscalls::{SyscallError, SyscallResult};
use crate::service::{Id, ServiceRef, WriteError};
use crate::util::address::VirtualAddress;

const WRITE_END_FLAG: u64 = 1;

fn map_write_error_to_syscall_error(err: WriteError) -> SyscallError {
    match err {
        WriteError::InvalidConnection => SyscallError::ResourceNotFound,
        WriteError::NoOpenRequest | WriteError::RequestClosed => SyscallError::RequestClosed,
        WriteError::ParameterOverflow => SyscallError::ParameterOverflow,
    }
}

pub fn write_syscall(args: &SyscallArgs, current_service: ServiceRef) -> SyscallResult {
    let connection_id = args.arg0 as Id;
    let buffer_size = args.arg1 as usize;
    let buffer_ptr = args.arg2;
    let flags = args.arg3;

    let Some(write_buffer) =
        atomic_block(|| current_service.deref_incoming_pointer(VirtualAddress::from(buffer_ptr)))
    else {
        return Err(SyscallError::InvalidPointerMappings);
    };

    if buffer_size > write_buffer.len() {
        return Err(SyscallError::InvalidStringArgument);
    }

    let source_buffer = &write_buffer[0..buffer_size];
    let mut start = 0;

    atomic_block(|| loop {
        let result = current_service.write(connection_id, source_buffer, start);

        match result {
            Err(err) => return Err(map_write_error_to_syscall_error(err)),
            Ok(written) => {
                start += written;

                if start > 0 || source_buffer.is_empty() {
                    if (flags & WRITE_END_FLAG) != 0 {
                        current_service
                            .close_write(connection_id)
                            .map_err(map_write_error_to_syscall_error)?;
                    }

                    return Ok(start as u64);
                }

                // because the buffer could not be written in its entirety, it must be full.
                // Therefore wait until the other side reads from the buffer.
                current_service.block_until_write_available(connection_id)
            }
        }
    })
}
