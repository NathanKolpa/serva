//! Typed wrappers around syscalls.

use core::arch::asm;
use core::ffi::CStr;
use core::fmt::Debug;
use core::mem::size_of;

use crate::{decode_syscall_result, SyscallError, SyscallResult};

type KernelSyscall = extern "C" fn(syscall: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> u64;

/// # Safety
///
/// This function is unsafe because arguments can be interpreted as pointers.
/// The caller must ensure that the rust borrow checker rule's are respected on order to guarantee safety.
pub unsafe fn syscall(syscall: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> SyscallResult {
    let segment: u16;
    asm!("mov {0:x}, cs", out(reg) segment, options(nomem, nostack, preserves_flags));

    let raw_result: u64;

    // Because the limitations of the x86_64 we can't use the `syscall` instruction while in the kernel privilege level.
    // Therefore we check if the last 2 bits of CS indicate a user privilege level.
    if (segment & 0b11) != 0 {
        asm!(
            "mov rax, 1",
            "mov rdi, 2",
            "mov rsi, 3",
            "mov rdx, 4",
            "mov r10, 5",
            "syscall",
            in("rax") syscall,
            in("rdi") arg0,
            in("rsi") arg1,
            in("rdx") arg2,
            in("r10") arg3,
        );

        asm!("", out("rax") raw_result, options(nomem, nostack, preserves_flags));
    } else {
        let kernel_syscall_location = 0x3fffffff000 as *const KernelSyscall;
        raw_result = (*kernel_syscall_location)(syscall, arg0, arg1, arg2, arg3);
    }

    decode_syscall_result(raw_result)
}

pub fn thread_exit() -> ! {
    todo!()
}

pub fn hello() {
    unsafe {
        let _ = syscall(0, 0, 0, 0, 0);
    }
}

type Handle = u16;
pub type ConnectionHandle = Handle;

pub type EndpointId = Handle;
pub type SpecId = Handle;

#[derive(Copy, Clone, Debug)]
pub enum ConnectError {
    OutOfMemory,
    ResourceNotFound,
}

fn unexpected_error(err: SyscallError) -> ! {
    panic!("Unexpected syscall error: {:?}", err)
}

pub fn connect(spec_name: &CStr) -> Result<ConnectionHandle, ConnectError> {
    let result = unsafe { syscall(1, spec_name.as_ptr() as u64, 0, 0, 0) };

    match result {
        Ok(id) => Ok(id as ConnectionHandle),
        Err(err) => match err {
            SyscallError::OutOfMemory => Err(ConnectError::OutOfMemory),
            SyscallError::ResourceNotFound => Err(ConnectError::ResourceNotFound),
            e => unexpected_error(e),
        },
    }
}

#[derive(Copy, Clone, Debug)]
pub enum RequestError {
    OperationNotPermitted,
    ResourceNotFound,
}

pub unsafe fn request(
    connection: ConnectionHandle,
    endpoint_name: &CStr,
) -> Result<(), RequestError> {
    let result = unsafe { syscall(2, connection as u64, endpoint_name.as_ptr() as u64, 0, 0) };

    match result {
        Ok(_) => Ok(()),
        Err(err) => match err {
            SyscallError::ResourceNotFound => Err(RequestError::ResourceNotFound),
            SyscallError::OperationNotPermitted => Err(RequestError::OperationNotPermitted),
            e => unexpected_error(e),
        },
    }
}

#[derive(Copy, Clone, Debug)]
pub enum WriteError {
    ResourceNotFound,
    RequestClosed,
    ParameterOverflow,
}

pub unsafe fn write(
    connection: ConnectionHandle,
    buffer: &[u8],
    end: bool,
) -> Result<usize, WriteError> {
    let mut flags = 0;
    flags |= (end as u64) << 0;

    let result = unsafe {
        syscall(
            3,
            connection as u64,
            buffer.as_ptr() as u64,
            buffer.len() as u64,
            flags,
        )
    };

    match result {
        Ok(b) => Ok(b as usize),
        Err(err) => match err {
            SyscallError::ParameterOverflow => Err(WriteError::ParameterOverflow),
            SyscallError::ResourceNotFound => Err(WriteError::ResourceNotFound),
            SyscallError::RequestClosed => Err(WriteError::RequestClosed),
            e => unexpected_error(e),
        },
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ReadError {
    ResourceNotFound,
}

pub unsafe fn read(connection: ConnectionHandle, buffer: &mut [u8]) -> Result<usize, ReadError> {
    let result = unsafe {
        syscall(
            4,
            connection as u64,
            buffer.as_ptr() as u64,
            buffer.len() as u64,
            0,
        )
    };

    match result {
        Ok(read) => Ok(read as usize),
        Err(err) => match err {
            SyscallError::ResourceNotFound => Err(ReadError::ResourceNotFound),
            e => unexpected_error(e),
        },
    }
}

/// # Safety
///
/// This function is unsafe to prevent unowned access to this global "resource"
pub unsafe fn accept() -> Option<(ConnectionHandle, EndpointId)> {
    let result = unsafe { syscall(5, 0, 0, 0, 0) };

    match result {
        Ok(data) => {
            let is_some = (data & (1 << (size_of::<Handle>() * 2 * 8))) != 0;

            if !is_some {
                return None;
            }

            let connection_id = data as ConnectionHandle;
            let endpoint_id = (data >> (size_of::<Handle>() * 8)) as Handle;

            Some((connection_id, endpoint_id))
        }
        Err(e) => unexpected_error(e),
    }
}

pub struct EndpointStat {
    pub id: EndpointId,
}

pub fn stat_endpoint(_spec_id: Option<SpecId>, endpoint_name: &CStr) -> Option<EndpointStat> {
    let result = unsafe { syscall(6, endpoint_name.as_ptr() as u64, 0, 0, 0) };

    match result {
        Ok(id) => Some(EndpointStat {
            id: id as EndpointId,
        }),
        Err(e) => match e {
            SyscallError::ResourceNotFound => None,
            e => unexpected_error(e),
        },
    }
}
