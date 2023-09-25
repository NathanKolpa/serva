use alloc::borrow::Cow;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::memory::MemoryMapper;
use crate::multi_tasking::scheduler::ThreadBlocker;
use crate::util::address::VirtualAddress;
use crate::util::collections::FixedVec;
use crate::util::sync::SpinMutex;

pub type Id = u32;
pub type CowString = Cow<'static, str>;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum Privilege {
    /// A service that runs directly in the kernel, a.k.a. Ring0
    Kernel = 2,

    /// A service that can manage userland without restrictions.
    System = 1,

    /// A service that can only use the resources specified in the [`ServiceSpec`].
    User = 0,
}

#[derive(Clone)]
pub enum ServiceEntrypoint {
    MappedFunction(VirtualAddress),
    Elf(), // TODO: place a request here!
}

pub enum EndpointParameter {
    SizedBuffer(u32),
    StreamHandle(u32),
    UnsizedBuffer,
}

impl EndpointParameter {
    fn type_id(&self) -> u64 {
        match self {
            EndpointParameter::SizedBuffer(_) => 0,
            EndpointParameter::StreamHandle(_) => 1,
            EndpointParameter::UnsizedBuffer => 2,
        }
    }

    fn data(&self) -> u64 {
        match self {
            EndpointParameter::SizedBuffer(size) => *size as u64,
            EndpointParameter::StreamHandle(handle) => *handle as u64,
            EndpointParameter::UnsizedBuffer => 0,
        }
    }
}

impl Into<u64> for EndpointParameter {
    fn into(self) -> u64 {
        (self.type_id() << (64 - 2)) | self.data()
    }
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

    pub service: Option<Id>,
    pub entrypoint: ServiceEntrypoint,
    pub discovery_allowed: bool,
}

pub struct Intent {
    pub source_spec_id: Id,
    pub endpoint_id: Id,
}

pub struct Endpoint {
    pub id: Id,
    pub spec_id: Id,
    pub name: CowString,
    pub min_privilege: Privilege,
    pub request: FixedVec<16, EndpointParameter>,
    pub response: FixedVec<16, EndpointParameter>,
}

pub struct Pipe {
    pub buffer: VecDeque<u8>,
    pub write_arg_index: u8,
    pub read_arg_index: u8,
    pub current_arg_written: usize,
    pub closed: bool,
    pub write_block: Option<ThreadBlocker>,
    pub read_block: Option<ThreadBlocker>,
}

impl Default for Pipe {
    fn default() -> Self {
        Self {
            read_arg_index: 0,
            write_arg_index: 0,
            current_arg_written: 0,
            write_block: None,
            read_block: None,
            closed: false,
            buffer: VecDeque::with_capacity(1024 * 2),
        }
    }
}

pub struct Connection {
    pub target_service: Id,
    pub current_request: Option<Request>,
    pub request: Pipe,
    pub response: Pipe,
}

pub struct Service {
    pub id: Id,
    pub spec_id: Id,
    pub connections: Vec<Arc<SpinMutex<Connection>>>,
    pub memory_map: MemoryMapper,
}

pub struct Request {
    pub endpoint_id: Id,
}
