use syscall::EndpointId;

#[derive(PartialEq, Eq)]
pub struct Endpoint {
    handle: EndpointId,
}

impl Endpoint {
    pub const unsafe fn from_handle(handle: EndpointId) -> Self {
        Self {
            handle
        }
    }

    pub fn lookup<E: AsRef<str>>(name: E) -> Option<Endpoint> {
        todo!()
    }
}