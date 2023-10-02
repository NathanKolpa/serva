use crate::ipc::Request;
use core::ffi::CStr;
use syscall::ConnectionHandle;

pub struct Connection {
    handle: ConnectionHandle,
}

impl Connection {
    pub const unsafe fn from_handle(handle: ConnectionHandle) -> Self {
        Self { handle }
    }

    pub fn handle(&self) -> ConnectionHandle {
        self.handle
    }

    pub fn request<E: AsRef<CStr>>(&mut self, endpoint: E) -> crate::io::Result<Request<'_>> {
        unsafe {
            syscall::request(self.handle, endpoint.as_ref())?;
            Ok(Request::from_handle(self.handle))
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        todo!()
    }
}
