use crate::ipc::request::Request;
use crate::ipc::Endpoint;

pub type EndpointHandler = fn(Request<'_>);

#[repr(C)]
pub struct Listener {
    _phantom: u8,
}

impl Listener {
    pub unsafe fn new() -> Self {
        Self { _phantom: 0 }
    }

    pub fn accept(&mut self) -> Option<(Request<'_>, Endpoint)> {
        unsafe {
            syscall::accept().map(|(c, e)| (Request::from_handle(c), Endpoint::from_handle(e)))
        }
    }
}
