use crate::io::{Read, Write};
use core::marker::PhantomData;
use syscall::ConnectionHandle;

pub struct Request<'a> {
    handle: ConnectionHandle,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Request<'a> {
    pub const unsafe fn from_handle(handle: ConnectionHandle) -> Self {
        Self {
            handle,
            _phantom: PhantomData,
        }
    }
}

impl Read for Request<'_> {
    fn read(&mut self, buf: &mut [u8]) -> crate::io::Result<usize> {
        unsafe { Ok(syscall::read(self.handle, buf)?) }
    }
}

impl Write for Request<'_> {
    fn write(&mut self, buf: &[u8]) -> crate::io::Result<usize> {
        unsafe { Ok(syscall::write(self.handle, buf, false)?) }
    }
}

impl Drop for Request<'_> {
    fn drop(&mut self) {
        unsafe {
            syscall::write(self.handle, &[], true).expect("request should be closable");
        }
    }
}
