use crate::service::Privilege;
use core::fmt::{Debug, Formatter};
use core::mem::{transmute, transmute_copy};

#[repr(u64)]
#[derive(Copy, Clone, Debug)]
pub enum SyscallError {
    /// The syscall number was not recognised as a valid syscall.
    UnknownSyscall = 1,

    OperationNotPermitted,

    OutOfMemory,

    ResourceNotFound,

    InvalidPointerMappings,

    InvalidStringArgument,

    ConnectionClosed,

    ConnectionBusy,

    NoOpenRequest,

    ParameterOverflow,
}

pub type SyscallResult = Result<u64, SyscallError>;

pub fn encode_result(result: SyscallResult) -> u64 {
    let binary_result: i64 = match result {
        Ok(v) => v as i64,
        Err(e) => -(e as i64),
    };

    binary_result as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    #[test_case]
    fn test_encode_ok_result() {
        let result = Ok(1);
        let encoded = encode_result(result);
        assert_eq!(1, encoded);
    }

    #[test_case]
    fn test_encode_ok_result() {
        let result = Ok(1);
        let encoded = encode_result(result);
        assert_eq!(1, encoded);
    }

    #[test_case]
    fn test_encode_err_result() {
        let result = Err(SyscallError::UnknownSyscall);
        let encoded = encode_result(result);
        assert_eq!(-1i64 as u64, encoded);
    }

    #[test_case]
    fn test_encode_err_as_negative() {
        let result = Err(SyscallError::UnknownSyscall);
        let encoded = encode_result(result);
        let signed = encoded as i64;

        assert!(signed < 0);
    }

    #[test_case]
    fn test_encode_ok_as_positive() {
        let result = Ok(0);
        let encoded = encode_result(result);
        let signed = encoded as i64;

        assert!(signed >= 0);
    }
}
