use core::ffi::CStr;

use crate::arch::x86_64::interrupts::atomic_block;
use crate::arch::x86_64::syscalls::SyscallArgs;
use crate::interface::syscalls::{SyscallError, SyscallResult};
use crate::service::{CreateRequestError, Id, ServiceRef};
use crate::util::address::VirtualAddress;

pub fn request_syscall(args: &SyscallArgs, current_service: ServiceRef) -> SyscallResult {
    let connection_id = args.arg0 as Id;

    let Some(target_endpoint_name) =
        atomic_block(|| current_service.deref_incoming_pointer(VirtualAddress::from(args.arg1)))
    else {
        return Err(SyscallError::InvalidPointerMappings);
    };

    let target_endpoint_name = CStr::from_bytes_until_nul(&target_endpoint_name[0..256])
        .map_err(|_| SyscallError::InvalidStringArgument)?
        .to_str()
        .map_err(|_| SyscallError::InvalidStringArgument)?;

    atomic_block(|| {
        let target_service = current_service
            .get_service_from_connection(connection_id)
            .ok_or(SyscallError::ConnectionClosed)?;

        let target_service_spec = target_service.spec();
        let target_endpoint = target_service_spec
            .get_endpoint_by_name(target_endpoint_name)
            .ok_or(SyscallError::ResourceNotFound)?;

        let result = current_service.create_request_to(connection_id, target_endpoint.id());

        match result {
            Ok(()) => Ok(0),
            Err(err) => match err {
                CreateRequestError::NotPermitted => {
                    return Err(SyscallError::OperationNotPermitted)
                }
                CreateRequestError::ConnectionBusy => return Err(SyscallError::ConnectionBusy),
                CreateRequestError::InvalidEndpointId => {
                    panic!("Expected the endpoint to be valid before creating the request")
                }
            },
        }
    })
}
