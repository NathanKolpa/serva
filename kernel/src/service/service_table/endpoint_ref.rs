use crate::service::model::Id;
use crate::service::{Privilege, ServiceTable};

pub struct EndpointRef<'a> {
    table: &'a ServiceTable,
    id: Id,
}

impl<'a> EndpointRef<'a> {
    pub fn new(table: &'a ServiceTable, spec_id: Id) -> Self {
        Self { table, id: spec_id }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn is_allowed(&self, privilege: Privilege) -> bool {
        let endpoints = self.table.endpoints.lock();
        let endpoint_privilege = endpoints[self.id as usize].min_privilege;

        privilege >= endpoint_privilege
    }
}
