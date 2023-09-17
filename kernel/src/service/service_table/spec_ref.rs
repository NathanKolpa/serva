use crate::service::model::Id;
use crate::service::{EndpointRef, ServiceTable};

pub struct ServiceSpecRef<'a> {
    table: &'a ServiceTable,
    id: Id,
}

impl<'a> ServiceSpecRef<'a> {
    pub fn new(table: &'a ServiceTable, spec_id: Id) -> Self {
        Self { table, id: spec_id }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn get_endpoint_by_name(&self, endpoint_name: &str) -> Option<EndpointRef> {
        let specs = self.table.specs.lock();
        let endpoints = self.table.endpoints.lock();
        let spec = &specs[self.id as usize];

        for endpoint in (spec.endpoints_start..spec.endpoints_end).map(|i| &endpoints[i as usize]) {
            if endpoint.name == endpoint_name {
                return Some(EndpointRef::new(self.table, endpoint.id));
            }
        }

        None
    }
}
