use syscall::EndpointId;

#[derive(PartialEq, Eq)]
pub struct Endpoint {
    handle: EndpointId,
}

impl Endpoint {
    pub const unsafe fn from_handle(handle: EndpointId) -> Self {
        Self { handle }
    }

    pub fn lookup<E: AsRef<str>>(name: E) -> Option<Self> {
        syscall::stat_endpoint(None, name.as_ref()).map(|s| unsafe { Self::from_handle(s.id) })
    }
}
