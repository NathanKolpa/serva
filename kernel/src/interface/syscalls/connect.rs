use crate::arch::x86_64::interrupts::atomic_block;
use crate::arch::x86_64::syscalls::SyscallArgs;
use crate::interface::syscalls::{SyscallError, SyscallResult, EXPECT_CURRENT_SERVICE};
use crate::memory::NewMappingError;
use crate::multi_tasking::scheduler::SCHEDULER;
use crate::service::{ConnectError, NewServiceError, ServiceRef, SERVICE_TABLE};

pub fn connect_syscall(args: &SyscallArgs) -> SyscallResult {
    atomic_block(|| {
        let source_service = SCHEDULER.current_service().expect(EXPECT_CURRENT_SERVICE);

        let target_spec = args.arg1 as usize;

        let result = SERVICE_TABLE.connect_to(source_service.id(), target_spec);

        match result {
            Ok(service) => Ok(service.id() as u64),
            Err(e) => match e {
                ConnectError::SpecDoesNotExist => Err(SyscallError::UnknownResource),
                ConnectError::FailedToStartService(s) => match s {
                    NewServiceError::FailedToCreateNewMemoryMap(e)
                    | NewServiceError::FailedToCreateStack(e) => match e {
                        NewMappingError::OutOfFrames => Err(SyscallError::OutOfMemory),
                        _ => panic!("Internal error while mapping new service {e:?}"),
                    },
                    NewServiceError::SpecNotFound => Err(SyscallError::UnknownResource),
                },
            },
        }
    })
}
