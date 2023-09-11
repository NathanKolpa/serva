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
}
