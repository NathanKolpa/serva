use crate::arch::x86_64::interrupts::atomic_block;
use crate::arch::x86_64::syscalls::SyscallArgs;
use crate::interface::syscalls::{SyscallError, SyscallResult, EXPECT_CURRENT_SERVICE};
use crate::multi_tasking::scheduler::SCHEDULER;
use crate::service::{Id, WriteError};
use crate::util::address::VirtualAddress;

pub fn request_syscall(args: &SyscallArgs) -> SyscallResult {
    let current_service =
        atomic_block(|| SCHEDULER.current_service().expect(EXPECT_CURRENT_SERVICE));

    let connection_id = args.arg0 as Id;
    let buffer_size = args.arg2 as usize;

    let Some(write_buffer) =
        atomic_block(|| current_service.deref_incoming_pointer(VirtualAddress::from(args.arg1)))
    else {
        return Err(SyscallError::InvalidPointerMappings);
    };

    let source_buffer = &write_buffer[0..buffer_size];
    let mut start = 0;

    atomic_block(|| loop {
        let result = current_service.write(connection_id, source_buffer, start);

        match result {
            Err(err) => {
                return match err {
                    WriteError::InvalidConnection => Err(SyscallError::ConnectionClosed),
                }
            }
            Ok(written) => {
                start += written;

                debug_println!("Check {} {}", buffer_size, written);

                if start >= source_buffer.len() {
                    return Ok(0)
                }

                debug_println!("Uhh {} {}", buffer_size, written);

                // because the buffer could not be written in its entirety, it must be full.
                // Therefore wait until the other side reads from the buffer.
                current_service.block_until_write_available(connection_id)
            }
        }
    })
}
