use crate::memory::MemoryMapper;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use crate::util::address::VirtualAddress;

pub type Id = usize;
pub type CowString = Cow<'static, str>;

#[derive(Debug, Copy, Clone)]
pub enum Privilege {
    /// A service that runs directly in the kernel, a.k.a. Ring0
    Kernel = 0,

    /// A service that can manage userland without restrictions.
    System,

    /// A service that can manage userland but in a protected way.
    User,

    /// A service that can only use the resources specified in the [`ServiceSpec`].
    Local,
}

#[derive(Clone)]
pub enum ServiceEntrypoint {
    MappedFunction(VirtualAddress),
    Elf() // TODO: place a request here!
}

pub struct ServiceSpec {
    pub id: Id,

    /// The addressable and unique name of the service.
    pub name: CowString,

    /// The service's privilege level.
    pub privilege: Privilege,

    pub intents_start: Id,
    pub intents_end: Id,

    pub endpoints_start: Id,
    pub endpoints_end: Id,

    pub entrypoint: ServiceEntrypoint,
}

pub struct Intent {
    pub source_spec_id: Id,
    pub target_spec_id: Id,
    pub endpoint_id: Id,
    pub required: bool,
}

pub struct Endpoint {
    pub id: Id,
    pub spec_id: Id,
    pub name: CowString,
    pub min_privilege: Privilege,
}

pub struct Connection {
    pub service_id: Id,
}

pub struct Service {
    pub id: Id,
    pub spec_id: Id,
    pub connections: Vec<Connection>,
    pub memory_map: MemoryMapper,
}
