use syscall::{SyscallError, SyscallResult};

use crate::arch::x86_64::interrupts::atomic_block;
use crate::arch::x86_64::syscalls::SyscallArgs;
use crate::memory::NewMappingError;
use crate::service::{ConnectError, NewServiceError, ServiceRef, SERVICE_TABLE};
use crate::util::address::VirtualAddress;

pub fn connect_syscall(args: &SyscallArgs, current_service: ServiceRef) -> SyscallResult {
    atomic_block(|| {
        let name_ptr = args.arg1;
        let name_len = args.arg0 as usize;

        let Some(target_spec_name) =
            current_service.deref_incoming_pointer(VirtualAddress::from(name_ptr))
        else {
            return Err(SyscallError::InvalidPointerMappings);
        };

        if name_len > target_spec_name.len() {
            return Err(SyscallError::InvalidPointerMappings);
        }

        let target_spec_name = core::str::from_utf8(&target_spec_name[0..name_len])
            .map_err(|_| SyscallError::InvalidStringArgument)?;

        let target_spec = SERVICE_TABLE
            .resolve_spec_name(target_spec_name)
            .ok_or(SyscallError::ResourceNotFound)?;

        let result = current_service.connect_to(target_spec.id());

        match result {
            Ok(connection_id) => Ok(connection_id as u64),
            Err(e) => match e {
                ConnectError::SpecDoesNotExist => Err(SyscallError::ResourceNotFound),
                ConnectError::FailedToStartService(s) => match s {
                    NewServiceError::FailedToCreateNewMemoryMap(e)
                    | NewServiceError::FailedToCreateStack(e) => match e {
                        NewMappingError::OutOfFrames => Err(SyscallError::OutOfMemory),
                        _ => panic!("Internal error while mapping new service {e:?}"),
                    },
                    NewServiceError::SpecNotFound => {
                        panic!("Internal error while starting new service")
                    }
                },
            },
        }
    })
}
