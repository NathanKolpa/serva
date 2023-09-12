use crate::arch::x86_64::interrupts::atomic_block;
use crate::arch::x86_64::syscalls::SyscallArgs;
use crate::interface::syscalls::{SyscallError, SyscallResult, EXPECT_CURRENT_SERVICE};
use crate::memory::NewMappingError;
use crate::multi_tasking::scheduler::SCHEDULER;
use crate::service::{ConnectError, NewServiceError, ServiceRef, SERVICE_TABLE};
use crate::util::address::VirtualAddress;
use core::ffi::{c_char, CStr};

pub fn connect_syscall(args: &SyscallArgs) -> SyscallResult {
    atomic_block(|| {
        let source_service = SCHEDULER.current_service().expect(EXPECT_CURRENT_SERVICE);

        let Some(target_spec_name) =
            source_service.deref_incoming_pointer(VirtualAddress::from(args.arg0))
        else {
            return Err(SyscallError::InvalidPointerMappings);
        };

        let target_spec_name = CStr::from_bytes_until_nul(&target_spec_name[0..256])
            .map_err(|_| SyscallError::InvalidStringArgument)?
            .to_str()
            .map_err(|_| SyscallError::InvalidStringArgument)?;

        let target_spec = SERVICE_TABLE
            .resolve_spec_name(target_spec_name)
            .ok_or(SyscallError::ResourceNotFound)?;

        let result = SERVICE_TABLE.connect_to(source_service.id(), target_spec.id());

        match result {
            Ok(service) => Ok(service.id() as u64),
            Err(e) => match e {
                ConnectError::SpecDoesNotExist => Err(SyscallError::ResourceNotFound),
                ConnectError::FailedToStartService(s) => match s {
                    NewServiceError::FailedToCreateNewMemoryMap(e)
                    | NewServiceError::FailedToCreateStack(e) => match e {
                        NewMappingError::OutOfFrames => Err(SyscallError::OutOfMemory),
                        _ => panic!("Internal error while mapping new service {e:?}"),
                    },
                    NewServiceError::SpecNotFound => Err(SyscallError::ResourceNotFound),
                },
            },
        }
    })
}
