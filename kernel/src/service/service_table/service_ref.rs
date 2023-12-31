use alloc::sync::Arc;
use core::cmp::min;
use core::fmt::{Debug, Formatter};
use core::ops::{Deref, DerefMut};
use essentials::address::VirtualAddress;
use essentials::collections::FixedVec;
use essentials::sync::SpinMutex;
use x86_64::paging::{PageTableEntryFlags, VirtualPage};

use crate::multi_tasking::scheduler::{ThreadBlocker, SCHEDULER};
use crate::service::model::{Connection, Endpoint, Id, Pipe, Request};
use crate::service::service_table::spec_ref::ServiceSpecRef;
use crate::service::{EndpointParameter, NewServiceError, Privilege, ServiceTable};

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
    NoOpenRequest,
    ParameterOverflow,
    RequestClosed,
}

#[derive(Debug)]
pub enum ReadError {
    InvalidConnection,
    RequestClosed,
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

    pub fn deref_incoming_pointer<'b>(&self, address: VirtualAddress) -> Option<&'b mut [u8]> {
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

        unsafe { Some(core::slice::from_raw_parts_mut(address.as_mut_ptr(), len)) }
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
        let handle = service.connections.len() as Id;

        let new_conn = Arc::new(SpinMutex::new(Connection {
            target_service: target_service.id(),
            current_request: None,
            request: Pipe::default(),
            response: Pipe::default(),
            request_close_block: None,
        }));

        service.connections.push(new_conn.clone());

        let target_service = &mut services[target_service.id as usize];
        target_service.connections.push(new_conn);

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

    pub fn read(
        &self,
        connection: Id,
        buffer: &mut [u8],
        start: usize,
    ) -> Result<usize, ReadError> {
        let total_len = buffer.len();
        let buffer = &mut buffer[start..total_len];

        let services = self.table.services.lock();
        let service = &services[self.id as usize];

        if connection as usize >= service.connections.len() {
            return Err(ReadError::InvalidConnection);
        }

        let mut conn = service.connections[connection as usize].lock();
        let pipe = self.get_read_pipe(conn.deref_mut());

        if pipe.buffer.is_empty() {
            return if pipe.closed {
                Err(ReadError::RequestClosed)
            } else {
                Ok(0)
            };
        }

        let read = min(buffer.len(), pipe.buffer.len());

        for i in 0..read {
            buffer[i] = pipe.buffer.pop_front().unwrap();
        }

        if !pipe.buffer.is_empty() {
            pipe.read_block = pipe.read_block.take().and_then(|b| b.unblock_one());
        }

        pipe.write_block = pipe.write_block.take().and_then(|b| b.unblock_one());

        // this is the last read call, we should clean up after ourselves.
        if pipe.buffer.is_empty() && pipe.closed {
            pipe.read_block = None;
            conn.request_close_block = None;
            conn.current_request = None;
        }

        Ok(read)
    }

    pub fn write(&self, connection: Id, buffer: &[u8], start: usize) -> Result<usize, WriteError> {
        let buffer = &buffer[start..buffer.len()];

        let services = self.table.services.lock();
        let service = &services[self.id as usize];

        if connection as usize >= service.connections.len() {
            return Err(WriteError::InvalidConnection);
        }

        let endpoints = self.table.endpoints.lock();
        let mut conn = service.connections[connection as usize].lock();

        let endpoint = conn
            .current_request
            .as_ref()
            .map(|req| &endpoints[req.endpoint_id as usize])
            .ok_or(WriteError::NoOpenRequest)?;

        let params = self.get_params(conn.deref(), endpoint);
        let pipe = self.get_write_pipe(conn.deref_mut());

        if pipe.closed {
            return Err(WriteError::RequestClosed);
        }

        let written = min(buffer.len(), pipe.buffer.capacity() - pipe.buffer.len());
        let write_iter = buffer[0..written].iter();

        let next_sizes = |index: usize| -> Result<Option<usize>, WriteError> {
            let current_param = params.get(index).ok_or(WriteError::ParameterOverflow)?;

            match current_param {
                EndpointParameter::SizedBuffer(size, _) => {
                    assert_ne!(*size, 0);
                    Ok(Some(*size as usize))
                }
                EndpointParameter::StreamHandle => todo!(),
                EndpointParameter::UnsizedBuffer => Ok(None),
            }
        };

        let mut max_size = next_sizes(pipe.write_arg_index as usize)?;

        for byte in write_iter {
            if let Some(max) = max_size {
                if pipe.current_arg_written + 1 > max {
                    pipe.write_arg_index += 1;
                    pipe.current_arg_written = 0;
                    max_size = next_sizes(pipe.write_arg_index as usize)?;
                }
            }

            pipe.buffer.push_back(*byte);
            pipe.current_arg_written += 1;
        }

        if pipe.buffer.len() < pipe.buffer.capacity() {
            pipe.write_block = pipe.write_block.take().and_then(|b| b.unblock_one());
        }

        pipe.read_block = pipe.read_block.take().and_then(|b| b.unblock_one());

        Ok(written)
    }

    pub fn close_write(&self, connection: Id) -> Result<(), WriteError> {
        let services = self.table.services.lock();
        let service = &services[self.id as usize];

        if connection as usize >= service.connections.len() {
            return Err(WriteError::InvalidConnection);
        }

        let mut conn = service.connections[connection as usize].lock();
        let pipe = self.get_write_pipe(conn.deref_mut());

        if pipe.closed {
            return Err(WriteError::RequestClosed);
        }

        pipe.closed = true;
        pipe.read_block = None;
        conn.request_close_block = None;
        conn.current_request = None;

        Ok(())
    }

    fn add_current_to_block(blocker: &mut Option<ThreadBlocker>) {
        match blocker {
            None => {
                *blocker = Some(SCHEDULER.block_current());
            }
            Some(block) => {
                block.block_current();
            }
        }
    }

    fn block_until_pipe_event(
        &self,
        connection: Id,
        pipe_selector: impl FnOnce(&mut Connection) -> &mut Pipe,
        block_selector: impl FnOnce(&mut Pipe) -> &mut Option<ThreadBlocker>,
    ) {
        {
            let services = self.table.services.lock();
            let service = &services[self.id as usize];

            let mut conn = service.connections[connection as usize].lock();
            let pipe = pipe_selector(conn.deref_mut());

            let blocker = block_selector(pipe);
            Self::add_current_to_block(blocker);
        }

        SCHEDULER.yield_current();
    }

    pub fn block_until_write_available(&self, connection: Id) {
        self.block_until_pipe_event(
            connection,
            |c| self.get_write_pipe(c),
            |p| &mut p.write_block,
        )
    }

    pub fn block_until_request_close(&self, connection: Id) {
        {
            let services = self.table.services.lock();
            let service = &services[self.id as usize];

            let mut conn = service.connections[connection as usize].lock();

            let blocker = &mut conn.request_close_block;
            Self::add_current_to_block(blocker);
        }

        SCHEDULER.yield_current();
    }

    pub fn block_until_read_available(&self, connection: Id) {
        self.block_until_pipe_event(connection, |c| self.get_read_pipe(c), |p| &mut p.read_block)
    }

    fn get_write_pipe<'b>(&self, connection: &'b mut Connection) -> &'b mut Pipe {
        if connection.target_service == self.id {
            &mut connection.response
        } else {
            &mut connection.request
        }
    }

    fn get_read_pipe<'b>(&self, connection: &'b mut Connection) -> &'b mut Pipe {
        if connection.target_service == self.id {
            &mut connection.request
        } else {
            &mut connection.response
        }
    }

    fn get_params<'b>(
        &self,
        connection: &Connection,
        endpoint: &'b Endpoint,
    ) -> &'b FixedVec<16, EndpointParameter> {
        if connection.target_service == self.id {
            &endpoint.response
        } else {
            &endpoint.request
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

        *current_request = Some(Request {
            endpoint_id,
            accepted: false,
        });

        let target_service_id = conn.target_service as usize;

        Self::reset_pipe(&mut conn.request);
        Self::reset_pipe(&mut conn.response);
        drop(conn);

        let target_service = &mut services[target_service_id];
        target_service.accept_block = target_service
            .accept_block
            .take()
            .and_then(|b| b.unblock_one());

        Ok(())
    }

    fn reset_pipe(pipe: &mut Pipe) {
        pipe.closed = false;
        pipe.current_arg_written = 0;
        pipe.buffer.clear();
        pipe.read_block = None;
        pipe.write_block = None;
        pipe.write_arg_index = 0;
        pipe.read_arg_index = 0;
        pipe.reading_closed = false;
    }

    pub fn accept_next_connection_request(&self) -> Option<(Id, Id)> {
        let mut services = self.table.services.lock();
        let service = &mut services[self.id as usize];

        for (id, connection) in service.connections.iter_mut().enumerate() {
            let mut connection = connection.lock();

            if let Some(req) = connection.current_request.as_mut() {
                if !req.accepted {
                    req.accepted = true;
                    return Some((id as Id, req.endpoint_id));
                }
            }
        }

        None
    }

    pub fn block_until_next_request(&self) {
        {
            let mut services = self.table.services.lock();
            let service = &mut services[self.id as usize];
            Self::add_current_to_block(&mut service.accept_block);
        }

        SCHEDULER.yield_current();
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
