mod service_ref;
mod spec_ref;

pub use service_ref::*;
pub use spec_ref::*;

use crate::arch::x86_64::paging::{PageSize, PageTableEntryFlags, VirtualPage};
use crate::memory::{MemoryMapper, NewMappingError};
use crate::multi_tasking::scheduler::{Thread, ThreadStack, SCHEDULER};
use crate::service::model::*;
use crate::service::service_table::spec_ref::ServiceSpecRef;
use crate::util::address::VirtualAddress;
use crate::util::sync::{PanicOnce, SpinMutex};
use alloc::vec::Vec;

#[derive(Debug)]
pub enum NewServiceError {
    FailedToCreateNewMemoryMap(NewMappingError),
    FailedToCreateStack(NewMappingError),
    SpecNotFound,
}

pub struct NewIntent {
    pub target_spec_id: Id,
    pub endpoint_id: Id,
    pub required: bool,
}

pub struct NewEndpoint {
    pub target_spec_id: Id,
    pub min_privilege: Privilege,
    pub name: CowString,
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
        entrypoint: ServiceEntrypoint,
        spec_intents: impl IntoIterator<Item = NewIntent>,
        spec_endpoints: impl IntoIterator<Item = NewEndpoint>,
    ) -> ServiceSpecRef<'_> {
        let mut specs = self.specs.lock();
        let mut intents = self.intents.lock();
        let mut endpoints = self.endpoints.lock();

        let new_spec_id = specs.len();

        let intents_start = intents.len();
        intents.extend(spec_intents.into_iter().map(|n| Intent {
            endpoint_id: n.endpoint_id,
            target_spec_id: n.target_spec_id,
            source_spec_id: new_spec_id,
            required: n.required,
        }));
        let intents_end = intents.len();

        // TODO: validate requirements.
        // TODO: validate that the name is unique.

        let endpoints_start = endpoints.len();
        endpoints.extend(
            spec_endpoints
                .into_iter()
                .enumerate()
                .map(|(i, n)| Endpoint {
                    id: endpoints_start + i,
                    spec_id: new_spec_id,
                    name: n.name,
                    min_privilege: n.min_privilege,
                }),
        );
        let endpoints_end = endpoints.len();

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
        });

        ServiceSpecRef::new(self, new_spec_id)
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
        let mut stack_page = VirtualPage::new(VirtualAddress::from_l4_index(9), size).prev();

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
            .get_mut(spec_id)
            .ok_or(NewServiceError::SpecNotFound)?;

        let mut memory_map = self
            .root_memory_map
            .new_mapper(true)
            .map_err(NewServiceError::FailedToCreateNewMemoryMap)?;

        let id = services.len();

        let stack = Self::create_stack(&mut memory_map, spec.privilege)
            .map_err(NewServiceError::FailedToCreateStack)?;

        services.push(Service {
            id,
            memory_map,
            spec_id,
            connections: Vec::new(),
        });

        spec.service = Some(id);

        let addr = match spec.entrypoint {
            ServiceEntrypoint::MappedFunction(addr) => addr,
            ServiceEntrypoint::Elf() => todo!(),
        };

        let main_thread = unsafe { Thread::start_new(None, stack, addr, Some(id)) };

        SCHEDULER.add_thread(main_thread);

        Ok(ServiceRef::new(self, id))
    }

    pub fn get_service_by_id(&self, id: Id) -> ServiceRef<'_> {
        ServiceRef::new(self, id)
    }
}

pub static SERVICE_TABLE: ServiceTable = ServiceTable::new();
