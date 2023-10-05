use crate::arch::x86_64::interrupts::atomic_block;
use crate::arch::x86_64::syscalls::SyscallArgs;
use crate::service::ServiceRef;
use crate::util::address::VirtualAddress;
use core::ffi::CStr;
use syscall::{SyscallError, SyscallResult};

// TODO: this should give more info if needed and be allowed to target other specs

pub fn stat_endpoint_syscall(args: &SyscallArgs, current_service: ServiceRef) -> SyscallResult {
    atomic_block(|| {
        let Some(endpoint_name) =
            current_service.deref_incoming_pointer(VirtualAddress::from(args.arg0))
        else {
            return Err(SyscallError::InvalidPointerMappings);
        };

        let endpoint_name = CStr::from_bytes_until_nul(&endpoint_name[0..256])
            .map_err(|_| SyscallError::InvalidStringArgument)?
            .to_str()
            .map_err(|_| SyscallError::InvalidStringArgument)?;

        let spec = current_service.spec();

        let endpoint = spec
            .get_endpoint_by_name(endpoint_name)
            .ok_or(SyscallError::ResourceNotFound)?;

        Ok(endpoint.id() as u64)
    })
}
