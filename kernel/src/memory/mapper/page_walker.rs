use crate::arch::x86_64::paging::*;
use crate::memory::MemoryMapper;
use crate::util::address::{PhysicalAddress, VirtualAddress};

pub enum WalkError {
    NotMapped,
    NotOwned,
}

pub struct PageWalker<'a> {
    mapper: &'a MemoryMapper,
    index: u8,
    address: VirtualAddress,
    next_table: PhysicalAddress,
    done: bool,
}

impl<'a> PageWalker<'a> {
    pub unsafe fn new(
        address: VirtualAddress,
        next_table: PhysicalAddress,
        mapper: &'a MemoryMapper,
    ) -> Self {
        Self {
            mapper,
            index: 0,
            address,
            next_table,
            done: false,
        }
    }
}

impl<'a> Iterator for PageWalker<'a> {
    type Item = Result<WalkEntry<&'a PageTableEntry>, WalkError>;

    fn next(&mut self) -> Option<Self::Item> {
        let page_level = 4 - self.index;

        if self.done {
            return None;
        }

        if page_level == 0 {
            return None;
        }

        let index = self.address.indices()[self.index as usize];
        self.index += 1;

        let table_ptr: *const PageTable = self
            .mapper
            .translate_table_frame(self.next_table)
            .as_mut_ptr();

        let table = unsafe { &*table_ptr };

        self.next_table = table[index as usize].addr();
        let entry = &table[index as usize];

        if !entry.flags().present() {
            self.done = true;
            return Some(Err(WalkError::NotMapped));
        }

        Some(Ok(WalkEntry {
            level: page_level,
            entry,
        }))
    }
}

pub struct MutPageWalker<'a> {
    mapper: &'a mut MemoryMapper,
    index: u8,
    next_table: PhysicalAddress,
    address: VirtualAddress,
    done: bool,
}

impl<'a> MutPageWalker<'a> {
    pub unsafe fn new(
        next_table: PhysicalAddress,
        address: VirtualAddress,
        mapper: &'a mut MemoryMapper,
    ) -> Self {
        Self {
            mapper,
            index: 0,
            next_table,
            address,
            done: false,
        }
    }
}

impl<'a> Iterator for MutPageWalker<'a> {
    type Item = Result<WalkEntry<&'a mut PageTableEntry>, WalkError>;

    fn next(&mut self) -> Option<Self::Item> {
        let page_level = 4 - self.index;

        if self.done {
            return None;
        }

        if page_level == 0 {
            return None;
        }

        let index = self.address.indices()[self.index as usize];
        self.index += 1;

        let table_ptr: *mut PageTable = self
            .mapper
            .translate_table_frame(self.next_table)
            .as_mut_ptr();
        let table = unsafe { &mut *table_ptr };

        self.next_table = table[index as usize].addr();
        let entry = &mut table[index as usize];

        if page_level == 4 {
            if entry.flags().borrowed() {
                self.done = true;
                return Some(Err(WalkError::NotOwned));
            }
        }

        if !entry.flags().present() {
            self.done = true;
            return Some(Err(WalkError::NotMapped));
        }

        if entry.flags().huge() {
            self.done = true;
        }

        Some(Ok(WalkEntry {
            level: page_level,
            entry,
        }))
    }
}

#[derive(Copy, Clone)]
pub struct WalkEntry<E> {
    level: u8,
    entry: E,
}

impl<T> WalkEntry<T> {
    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn value(&self) -> &T {
        &self.entry
    }

    pub fn value_mut(&mut self) -> &mut T {
        &mut self.entry
    }
}
