use crate::service::model::Id;
use crate::service::ServiceTable;

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
}
