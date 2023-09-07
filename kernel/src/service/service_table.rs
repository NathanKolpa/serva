use alloc::vec::Vec;
use crate::service::model::*;
use crate::util::sync::SpinMutex;

pub struct ServiceTable {
    services: SpinMutex<Vec<Service>>
}

impl ServiceTable {
    pub fn register(&self, spec: ServiceSpec) {
        let mut lock = self.services.lock();

        let id = lock.len();

        lock.push(Service::new(id, spec));

    }
}