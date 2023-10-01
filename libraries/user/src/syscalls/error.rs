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

    RequestClosed,

    ParameterOverflow,
}

pub type SyscallResult = Result<u64, SyscallError>;