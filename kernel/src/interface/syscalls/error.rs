use core::fmt::{Debug, Formatter};
use core::mem::{transmute, transmute_copy};
use crate::service::Privilege;

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum SyscallError {
    /// The syscall number was not recognised as a valid syscall.
    UnknownSyscall = 1,

    InsufficientPrivilege(Privilege),
}

impl Debug for SyscallError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            SyscallError::UnknownSyscall => write!(f, "Unknown syscall (code 1)")?,
            SyscallError::InsufficientPrivilege(p) => write!(f, "Insufficient Privilege ({p:?} was required, code 2)")?
        }

        Ok(())
    }
}

pub type SyscallResult = Result<u64, SyscallError>;

pub fn encode_result(result: SyscallResult) -> u64 {
    let binary_result: i64 = match result {
        Ok(v) => v as i64,
        Err(e) => {
            let bin = unsafe { transmute::<_, u16>(e) };
            -(bin as i64)
        },
    };

    binary_result as u64
}

#[cfg(test)]
mod tests {
    use core::mem::size_of;
    use super::*;

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

    #[test_case]
    fn test_error_size() {
        // u16 may grow to u64, but the `encode` function has be updated accordingly
        assert_eq!(size_of::<SyscallError>(), size_of::<u16>());
    }
}