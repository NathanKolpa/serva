pub enum IoError {
    WriteError(syscall::WriteError),
    ReadError(syscall::ReadError),
    RequestError(syscall::RequestError),
}

impl From<syscall::WriteError> for IoError {
    fn from(value: syscall::WriteError) -> Self {
        Self::WriteError(value)
    }
}

impl From<syscall::ReadError> for IoError {
    fn from(value: syscall::ReadError) -> Self {
        Self::ReadError(value)
    }
}

impl From<syscall::RequestError> for IoError {
    fn from(value: syscall::RequestError) -> Self {
        Self::RequestError(value)
    }
}

pub type Result<T> = core::result::Result<T, IoError>;
