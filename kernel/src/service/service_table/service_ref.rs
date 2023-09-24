use crate::arch::x86_64::paging::{PageTableEntryFlags, VirtualPage};
use crate::multi_tasking::scheduler::{ThreadBlocker, SCHEDULER};
use crate::service::model::{Connection, Id, Pipe, Request, ServiceSpec};
use crate::service::service_table::spec_ref::ServiceSpecRef;
use crate::service::{NewServiceError, Privilege, ServiceTable};
use crate::util::address::VirtualAddress;
use crate::util::sync::SpinMutex;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::cmp::min;
use core::fmt::{Debug, Formatter};
use core::ops::DerefMut;

#[derive(Debug)]
pub enum ConnectError {
    SpecDoesNotExist,
    FailedToStartService(NewServiceError),
}

#[derive(Debug)]
pub enum CreateRequestError {
    InvalidEndpointId,
    ConnectionBusy,
    NotPermitted,
}

#[derive(Debug)]
pub enum WriteError {
    InvalidConnection,
}

pub struct ServiceRef<'a> {
    table: &'a ServiceTable,
    id: Id,
}

impl<'a> ServiceRef<'a> {
    pub fn new(table: &'a ServiceTable, service_id: Id) -> Self {
        Self {
            table,
            id: service_id,
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn set_memory_map_active(&self) {
        let services = self.table.services.lock();
        services[self.id as usize].memory_map.set_active()
    }

    pub fn deref_incoming_pointer<'b>(&self, address: VirtualAddress) -> Option<&'b [u8]> {
        let specs = self.table.specs.lock();
        let services = self.table.services.lock();
        let service = &services[self.id as usize];
        let spec = &specs[service.spec_id as usize];

        let is_page_safe = |flags: PageTableEntryFlags| -> bool {
            match spec.privilege {
                Privilege::Kernel => flags.present(),
                _ => flags.present() && flags.user_accessible(),
            }
        };

        let (flags, size) = service.memory_map.effective_flags(address)?;
        let page = VirtualPage::new(address, size);

        if !is_page_safe(flags) {
            return None;
        }

        let mut len = (page.end_addr() - address).as_usize();

        if let Some((next_flags, next_size)) = service.memory_map.effective_flags(address) {
            if is_page_safe(next_flags) {
                len += next_size.as_usize();
            }
        }

        unsafe { Some(core::slice::from_raw_parts(address.as_ptr(), len)) }
    }

    pub fn connect_to(&self, target_spec: Id) -> Result<Id, ConnectError> {
        let specs = self.table.specs.lock();

        let src = specs
            .get(target_spec as usize)
            .ok_or(ConnectError::SpecDoesNotExist)?;

        let target_service = match src.service {
            Some(service_id) => ServiceRef::new(self.table, service_id),
            None => {
                drop(specs);
                self.table
                    .start_service(target_spec)
                    .map_err(ConnectError::FailedToStartService)?
            }
        };

        let mut services = self.table.services.lock();
        let service = &mut services[self.id as usize];
        let handle = service.connections.len() as u32;

        let new_conn = Arc::new(SpinMutex::new(Connection {
            target_service: target_service.id(),
            current_request: None,
            request: Pipe::default(),
            response: Pipe::default(),
        }));

        service.connections.push(new_conn.clone());

        services[target_service.id as usize]
            .connections
            .push(new_conn);

        Ok(handle)
    }

    pub fn get_service_from_connection(&self, connection_id: Id) -> Option<ServiceRef> {
        let services = self.table.services.lock();
        let service = &services[self.id as usize];

        return service
            .connections
            .get(connection_id as usize)
            .map(|conn| ServiceRef {
                id: conn.lock().target_service,
                table: self.table,
            });
    }

    pub fn write(&self, connection: Id, buffer: &[u8], start: usize) -> Result<usize, WriteError> {
        let buffer = &buffer[start..buffer.len()];

        let services = self.table.services.lock();
        let service = &services[self.id as usize];

        if connection as usize >= service.connections.len() {
            return Err(WriteError::InvalidConnection);
        }

        let mut conn = service.connections[connection as usize].lock();
        let pipe = self.get_write_pipe(conn.deref_mut());

        let written = min(buffer.len(), pipe.buffer.capacity() - pipe.buffer.len());
        let write_buffer = &buffer[0..written];

        pipe.buffer.extend(write_buffer);
        Ok(written)
    }

    pub fn block_until_write_available(&self, connection: Id) {
        {
            let services = self.table.services.lock();
            let service = &services[self.id as usize];

            let mut conn = service.connections[connection as usize].lock();
            let pipe = self.get_write_pipe(conn.deref_mut());

            match &mut pipe.write_block {
                None => {
                    pipe.write_block = Some(SCHEDULER.block_current());
                }
                Some(block) => {
                    block.block_current();
                }
            }
        }

        SCHEDULER.yield_current();
    }

    fn get_write_pipe<'b>(&self, connection: &'b mut Connection) -> &'b mut Pipe {
        if connection.target_service == self.id {
            &mut connection.response
        } else {
            &mut connection.request
        }
    }

    pub fn spec(&self) -> ServiceSpecRef {
        let services = self.table.services.lock();
        let service = &services[self.id as usize];
        ServiceSpecRef::new(self.table, service.spec_id)
    }

    pub fn create_request_to(
        &self,
        connection_id: Id,
        endpoint_id: Id,
    ) -> Result<(), CreateRequestError> {
        // TODO: check if the endpoint id is valid for the connection.

        let mut services = self.table.services.lock();
        let specs = self.table.specs.lock();
        let service = &mut services[self.id as usize];
        let spec = &specs[service.spec_id as usize];

        let intents = self.table.intents.lock();
        let satisfying_intent = (spec.intents_start..spec.intents_end)
            .map(|id| &intents[id as usize])
            .find(|intent| intent.endpoint_id == endpoint_id);

        if satisfying_intent.is_none() {
            return Err(CreateRequestError::NotPermitted);
        }

        let mut conn = service.connections[connection_id as usize].lock();

        let current_request = &mut conn.current_request;
        if current_request.is_some() {
            return Err(CreateRequestError::ConnectionBusy);
        }

        *current_request = Some(Request { endpoint_id });

        Ok(())
    }
}

impl Debug for ServiceRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let lock = self.table.services.lock();

        let service = &lock[self.id as usize];

        f.debug_struct("Service")
            .field("id", &service.id)
            .field("spec_id", &service.spec_id)
            .field("open_connections", &service.connections.len())
            .finish()
    }
}
