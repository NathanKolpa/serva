use crate::service::model::Id;
use crate::service::ServiceTable;

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
}
