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

pub struct Endpoint {
    id: usize,
    name: String,
    min_privilege: Privilege,
    handler: VirtualAddress,
}

pub struct Interface {}

pub struct Intent {
    service: usize,
    endpoint: usize,
    required: bool,
}

pub struct Links {
    setup: VirtualAddress,
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
