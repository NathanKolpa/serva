use crate::interface::syscalls::{SyscallError, SyscallResult};
use crate::service::{CreateRequestError, Id, ServiceRef};
use essentials::address::VirtualAddress;
use x86_64::interrupts::atomic_block;
use x86_64::syscalls::SyscallArgs;

pub fn request_syscall(args: &SyscallArgs, current_service: ServiceRef) -> SyscallResult {
    let connection_id = args.arg0 as Id;
    let name_len = args.arg1 as usize;
    let name_ptr = args.arg2;

    let Some(target_endpoint_name) =
        atomic_block(|| current_service.deref_incoming_pointer(VirtualAddress::from(name_ptr)))
    else {
        return Err(SyscallError::InvalidPointerMappings);
    };

    if name_len > target_endpoint_name.len() {
        return Err(SyscallError::InvalidStringArgument);
    }

    let target_endpoint_name = core::str::from_utf8(&target_endpoint_name[0..name_len])
        .map_err(|_| SyscallError::InvalidStringArgument)?;

    atomic_block(|| {
        let target_service = current_service
            .get_service_from_connection(connection_id)
            .ok_or(SyscallError::ResourceNotFound)?;

        let target_service_spec = target_service.spec();
        let target_endpoint = target_service_spec
            .get_endpoint_by_name(target_endpoint_name)
            .ok_or(SyscallError::ResourceNotFound)?;

        loop {
            let result = current_service.create_request_to(connection_id, target_endpoint.id());

            match result {
                Ok(()) => return Ok(0),
                Err(err) => match err {
                    CreateRequestError::NotPermitted => {
                        return Err(SyscallError::OperationNotPermitted)
                    }
                    CreateRequestError::ConnectionBusy => {}
                    CreateRequestError::InvalidEndpointId => {
                        panic!("Expected the endpoint to be valid before creating the request")
                    }
                },
            }

            current_service.block_until_request_close(connection_id);
        }
    })
}
