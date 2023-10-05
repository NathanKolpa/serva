use alloc::vec::Vec;

pub use endpoint_ref::*;
use essentials::address::VirtualAddress;
use essentials::collections::FixedVec;
use essentials::sync::{PanicOnce, SpinMutex};
pub use service_ref::*;
pub use spec_ref::*;
use x86_64::paging::{PageSize, PageTableEntryFlags, VirtualPage};

use crate::memory::{MemoryMapper, NewMappingError};
use crate::multi_tasking::scheduler::{Thread, ThreadStack, SCHEDULER};
use crate::service::model::*;
use crate::service::service_table::spec_ref::ServiceSpecRef;

mod endpoint_ref;
mod service_ref;
mod spec_ref;

#[derive(Debug)]
pub enum NewServiceError {
    FailedToCreateNewMemoryMap(NewMappingError),
    FailedToCreateStack(NewMappingError),
    SpecNotFound,
}

#[derive(Debug)]
pub enum NewSpecError {
    NameTaken,
    RequirementsNotMet,
}

#[derive(Clone)]
pub struct NewIntent {
    pub spec_name: CowString,
    pub endpoint_name: CowString,
    pub required: bool,
}

pub struct NewEndpoint {
    pub min_privilege: Privilege,
    pub name: CowString,
    pub request: FixedVec<16, EndpointParameter>,
    pub response: FixedVec<16, EndpointParameter>,
}

pub struct ServiceTable {
    specs: SpinMutex<Vec<ServiceSpec>>,
    intents: SpinMutex<Vec<Intent>>,
    endpoints: SpinMutex<Vec<Endpoint>>,
    root_memory_map: PanicOnce<MemoryMapper>,
    services: SpinMutex<Vec<Service>>,
}

impl ServiceTable {
    pub const fn new() -> Self {
        Self {
            specs: SpinMutex::new(Vec::new()),
            intents: SpinMutex::new(Vec::new()),
            endpoints: SpinMutex::new(Vec::new()),
            root_memory_map: PanicOnce::new(),
            services: SpinMutex::new(Vec::new()),
        }
    }

    pub fn set_root_memory_map(&self, memory_map: MemoryMapper) {
        self.root_memory_map.initialize_with(memory_map);
    }

    /// Register a service spec, which serves as a factory.
    ///
    /// # Safety
    ///
    /// When `privilege` is equal to `Privilege::Kernel`
    /// then the entrypoint must point to valid and safe code.
    /// There is no reasonable way to prevent UB because usually the entrypoint is user input,
    /// so we buy the ticket, and take the ride.
    pub unsafe fn register_spec(
        &self,
        name: CowString,
        privilege: Privilege,
        discovery_allowed: bool,
        entrypoint: ServiceEntrypoint,
        spec_intents: impl IntoIterator<Item = NewIntent>,
        spec_endpoints: impl IntoIterator<Item = NewEndpoint>,
    ) -> Result<ServiceSpecRef<'_>, NewSpecError> {
        if self.resolve_spec_name(&name).is_some() {
            return Err(NewSpecError::NameTaken);
        }

        let specs = self.specs.lock();
        let new_spec_id = specs.len() as Id;
        drop(specs);

        let mut intents = self.intents.lock();

        let intents_start = intents.len() as Id;
        for new_intent in spec_intents {
            let endpoint = self.resolve_new_intent_to_endpoint(privilege, &new_intent);

            if let Some(endpoint) = endpoint {
                intents.push(Intent {
                    endpoint_id: endpoint,
                    source_spec_id: new_spec_id,
                });
            } else if new_intent.required {
                return Err(NewSpecError::RequirementsNotMet);
            }
        }
        let intents_end = intents.len() as Id;

        let mut endpoints = self.endpoints.lock();
        let endpoints_start = endpoints.len() as Id;
        endpoints.extend(
            spec_endpoints
                .into_iter()
                .enumerate()
                .map(|(i, n)| Endpoint {
                    id: endpoints_start + i as Id,
                    spec_id: new_spec_id,
                    name: n.name,
                    min_privilege: n.min_privilege,
                    request: n.request,
                    response: n.response,
                }),
        );
        let endpoints_end = endpoints.len() as Id;

        let mut specs = self.specs.lock();
        specs.push(ServiceSpec {
            id: new_spec_id,
            name,
            privilege,
            intents_start,
            intents_end,
            endpoints_start,
            endpoints_end,
            entrypoint,
            service: None,
            discovery_allowed,
        });

        Ok(ServiceSpecRef::new(self, new_spec_id))
    }

    fn resolve_new_intent_to_endpoint(
        &self,
        privilege: Privilege,
        intent: &NewIntent,
    ) -> Option<Id> {
        let spec = self.resolve_spec_name(&intent.spec_name)?;
        let endpoint = spec.get_endpoint_by_name(&intent.endpoint_name)?;

        if !endpoint.is_allowed(privilege) {
            return None;
        }

        Some(endpoint.id())
    }

    pub fn resolve_spec_name(&self, name: &str) -> Option<ServiceSpecRef> {
        self.specs
            .lock()
            .iter()
            .find(|spec| spec.name == name)
            .map(|spec| ServiceSpecRef::new(self, spec.id))
    }

    fn create_stack(
        mapper: &mut MemoryMapper,
        privilege: Privilege,
    ) -> Result<ThreadStack, NewMappingError> {
        let size = PageSize::Size4Kib;
        let initial_pages = 4;

        // begin on the last entry from the l4 index 8
        let stack_page = VirtualPage::new(VirtualAddress::from_l4_index(9), size).prev();

        let flags = match privilege {
            Privilege::Kernel => {
                let mut flags = PageTableEntryFlags::default();
                flags.set_present(true);
                flags.set_writable(true);
                flags
            }
            _ => {
                let mut flags = PageTableEntryFlags::default();
                flags.set_present(true);
                flags.set_writable(true);
                flags.set_user_accessible(true);
                flags
            }
        };

        let mut current_page = stack_page;
        for _ in 0..initial_pages {
            // we discard the cache flush since its not mapped.
            let _ = mapper.new_map(flags, flags, current_page)?;
            current_page = current_page.prev();
        }

        Ok(ThreadStack::from_page(stack_page))
    }

    pub fn start_service(&self, spec_id: Id) -> Result<ServiceRef, NewServiceError> {
        let mut services = self.services.lock();
        let mut specs = self.specs.lock();

        let spec = specs
            .get_mut(spec_id as usize)
            .ok_or(NewServiceError::SpecNotFound)?;

        let mut memory_map = self
            .root_memory_map
            .new_mapper(true)
            .map_err(NewServiceError::FailedToCreateNewMemoryMap)?;

        let id = services.len() as Id;

        let stack = Self::create_stack(&mut memory_map, spec.privilege)
            .map_err(NewServiceError::FailedToCreateStack)?;

        services.push(Service {
            id,
            memory_map,
            spec_id,
            connections: Vec::new(),
            accept_block: None,
        });

        spec.service = Some(id);

        let addr = match spec.entrypoint {
            ServiceEntrypoint::MappedFunction(addr) => addr,
            ServiceEntrypoint::Elf() => todo!(),
        };

        let main_thread = unsafe { Thread::start_new(Some("Main"), stack, addr, Some(id)) };

        SCHEDULER.add_thread(main_thread);

        Ok(ServiceRef::new(self, id))
    }

    pub fn get_service_by_id(&self, id: Id) -> ServiceRef<'_> {
        ServiceRef::new(self, id)
    }
}

pub static SERVICE_TABLE: ServiceTable = ServiceTable::new();
