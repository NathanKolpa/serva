use alloc::string::String;
use alloc::vec::Vec;

use crate::util::address::VirtualAddress;

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

pub enum SharedData {
    SizedBuffer { bytes: usize },
    SizedBufferList { element_size: usize},
    UnsizedBuffer,
    StreamHandle { handle: u64 },
}

pub struct EndpointParameter {
    name: Option<String>,
    data: SharedData,
}

pub struct EndpointParameterList {
    parameters: Vec<EndpointParameter>
}

pub struct Endpoint {
    id: usize,
    name: String,
    arguments: EndpointParameterList,
    response: Option<EndpointParameterList>,
    min_privilege: Privilege,
}

pub struct Interface {
    endpoints: Vec<Endpoint>
}

pub struct Intent {
    service: usize,
    endpoint: usize,
    required: bool,
}

pub struct Links {
    setup: VirtualAddress,
    endpoint_handlers: Vec<VirtualAddress>
}

pub struct Session {
    id: usize,
    source: usize,
    target: usize,
}

pub struct Request {
    id: usize,
    session: usize,
    endpoint: usize
}

pub struct ServiceSpec {
    /// The addressable and unique name of the service.
    name: String,

    /// The service's privilege level.
    privilege: Privilege,

    interface: Interface,

    /// These requirements have to be fulfilled before a service can be initialized.
    ///
    /// - When a service's `privilege` = [`Privilege::Local`], then the service can *only* request from the intended endpoints.
    /// - When a service's `privilege` = [`Privilege::User`] or higher, then the service can make requests to any "unintended" endpoints.
    intents: Vec<Intent>,
}

pub struct Service {
    id: usize,
    spec: ServiceSpec,
    running: bool,

    /// The link between the `ServiceSpec` and the actual code addresses.
    links: Option<Links>,
}

impl Service {
    pub fn new(id: usize, spec: ServiceSpec) -> Self {
        Self { id, spec, running: false, links: None }
    }
}
