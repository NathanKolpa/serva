use crate::arch::x86_64::interrupts::atomic_block;
use crate::arch::x86_64::syscalls::SyscallArgs;
use crate::service::ServiceRef;
use crate::util::address::VirtualAddress;
use syscall::{SyscallError, SyscallResult};

// TODO: this should give more info if needed and be allowed to target other specs

pub fn stat_endpoint_syscall(args: &SyscallArgs, current_service: ServiceRef) -> SyscallResult {
    atomic_block(|| {
        let name_len = args.arg0 as usize;
        let name_ptr = args.arg1;

        let Some(endpoint_name) =
            current_service.deref_incoming_pointer(VirtualAddress::from(name_ptr))
        else {
            return Err(SyscallError::InvalidPointerMappings);
        };

        if name_len > endpoint_name.len() {
            return Err(SyscallError::InvalidStringArgument);
        }

        let endpoint_name = core::str::from_utf8(&endpoint_name[0..name_len])
            .map_err(|_| SyscallError::InvalidStringArgument)?;

        let spec = current_service.spec();

        let endpoint = spec
            .get_endpoint_by_name(endpoint_name)
            .ok_or(SyscallError::ResourceNotFound)?;

        Ok(endpoint.id() as u64)
    })
}
