#[derive(Copy, Clone, Debug)]
pub enum SyscallError {
    /// The syscall id was not recognised as a valid syscall.
    UnknownSyscall = 1,

    /// The operation was denied based on the calling service's privilege level
    OperationNotPermitted,

    /// The operation required more memory than was available.
    OutOfMemory,

    /// The specified resource could not be identified.
    ///
    /// A resource may refer to one of the following:
    ///-  connection
    /// - endpoint
    /// - service
    /// - spec
    ResourceNotFound,

    /// A pointer argument was passed and was not mapped or mapped to a privileged location.
    InvalidPointerMappings,

    /// A string argument was passed and contains invalid characters or is not correctly null-terminated.
    InvalidStringArgument,

    /// The operation could not be completed because the request is closed.
    RequestClosed,

    /// The operation tried to write or read more bytes than was allowed by the endpoint parameters.
    ParameterOverflow,
}