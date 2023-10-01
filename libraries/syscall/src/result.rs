use core::mem::transmute_copy;

use crate::SyscallError;

pub type SyscallResult = Result<u64, SyscallError>;

pub fn encode_syscall_result(result: SyscallResult) -> u64 {
    let binary_result: i64 = match result {
        Ok(v) => v as i64,
        Err(e) => -(e as i64),
    };

    binary_result as u64
}

pub unsafe fn decode_syscall_result(result: u64) -> SyscallResult {
    let result = result as i64;
    let decoded;

    if result < 0 {
        let result = -result;
        decoded = Err(transmute_copy(&result));
    } else {
        decoded = Ok(result as u64);
    }

    decoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_encode_ok_result() {
        let result = Ok(1);
        let encoded = encode_syscall_result(result);
        assert_eq!(1, encoded);
    }

    #[test_case]
    fn test_encode_ok_result() {
        let result = Ok(1);
        let encoded = encode_syscall_result(result);
        assert_eq!(1, encoded);
    }

    #[test_case]
    fn test_encode_err_result() {
        let result = Err(SyscallError::UnknownSyscall);
        let encoded = encode_syscall_result(result);
        assert_eq!(-1i64 as u64, encoded);
    }

    #[test_case]
    fn test_encode_err_as_negative() {
        let result = Err(SyscallError::UnknownSyscall);
        let encoded = encode_syscall_result(result);
        let signed = encoded as i64;

        assert!(signed < 0);
    }

    #[test_case]
    fn test_encode_ok_as_positive() {
        let result = Ok(0);
        let encoded = encode_syscall_result(result);
        let signed = encoded as i64;

        assert!(signed >= 0);
    }
}
