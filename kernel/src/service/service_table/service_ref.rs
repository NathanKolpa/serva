use core::fmt::{Debug, Formatter};
use crate::service::model::Id;
use crate::service::{Privilege, ServiceTable};
use crate::util::address::VirtualAddress;

pub struct ServiceRef<'a> {
    table: &'a ServiceTable,
    id: Id,
}

impl<'a> ServiceRef<'a> {
    pub fn new(table: &'a ServiceTable, service_ref: Id) -> Self {
        Self { table, id: service_ref }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn set_memory_map_active(&self) {
        let services = self.table.services.lock();
        services[self.id].memory_map.set_active()
    }

    pub fn is_pointer_safe_to_deref(&self, address: VirtualAddress) -> bool {
        let specs = self.table.specs.lock();
        let services = self.table.services.lock();
        let service = &services[self.id];

        match specs[service.spec_id].privilege {
            Privilege::Kernel => return true,
            _ => {}
        }

        // check for present and user accessible
        todo!()
    }
}

impl Debug for ServiceRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let lock = self.table.services.lock();

        let service = &lock[self.id];

        f.debug_struct("Service")
            .field("id", &service.id)
            .field("spec_id", &service.spec_id)
            .field("open_connections", &service.connections.len())
            .finish()
    }
}