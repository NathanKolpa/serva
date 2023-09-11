use core::fmt::Display;

use page_walker::*;

use crate::arch::x86_64::paging::*;
use crate::memory::flush::{TableCacheFlush, TableListCacheFlush};
use crate::memory::frame_allocator::FrameAllocator;
use crate::memory::mapper::tree_display::MemoryMapTreeDisplay;
use crate::util::address::*;

mod page_walker;
mod tree_display;

#[derive(Debug, Clone, Copy)]
pub enum NewMappingError {
    AlreadyMapped,
    OutOfFrames,
    NotOwned,
}

#[derive(Debug, Clone, Copy)]
pub enum ModifyMappingError {
    NotOwned,
    NotMapped,
}

impl From<WalkError> for ModifyMappingError {
    fn from(value: WalkError) -> Self {
        match value {
            WalkError::NotMapped => Self::NotMapped,
            WalkError::NotOwned => Self::NotOwned,
        }
    }
}

/// The `MemoryMapper` struct manages the mappings between physical and virtual addresses.
///
/// # Use-case
///
/// There can be multiple instances of a memory mapper throughout the kernel.
/// This is because a `MemoryMapper` only manages a single level 4 page table.
/// Typically at startup, the kernel will setup its own memory map, and from which each new (user) task will have these kernel mappings shared in its own address space.
/// Having the kernel mapped in each running task will make `syscalls` that much more performant, since so page tables have to be swapped out.
///
/// # Ownership
///
/// To avoid programming errors and deallocate unused frames, the `MemoryMapper` mimics rust's ownership rules.
/// You can read more on the [`MemoryMapper::borrow_to_new_mapper`]'s documentation.
pub struct MemoryMapper {
    frame_allocator: &'static FrameAllocator,
    l4_page: PhysicalPage,
    global_offset: u64,
}

impl MemoryMapper {
    /// Create a new memory mapper instance.
    ///
    /// ## Safety
    /// The caller must guarantee that:
    /// 1. There is only one mapper at a given time.
    /// 2. The complete physical memory is mapped to virtual memory at the passed `global_offset`.
    /// 3. The passed `l4_page` points to a valid level 4 page.
    /// 4. The memory mapping are not owned by other `MemoryMapper` instances.
    pub unsafe fn new(
        frame_allocator: &'static FrameAllocator,
        l4_page: PhysicalPage,
        global_offset: u64,
    ) -> Self {
        Self {
            frame_allocator,
            l4_page,
            global_offset,
        }
    }

    fn translate_table_frame(&self, phys: PhysicalAddress) -> VirtualAddress {
        let a = phys.as_u64() + self.global_offset;
        a.into()
    }

    /// Get the physical address from a virtual address.
    pub fn translate_virtual_to_physical(&self, addr: VirtualAddress) -> Option<PhysicalAddress> {
        let last_entry = self.walk_entries(addr).last()?.ok()?;

        let offset = match last_entry.level() {
            1 => addr.page_offset(),
            2 => addr.l2_page_offset(),
            3 => addr.l3_page_offset(),
            _ => panic!("Unexpected level"),
        };

        Some(last_entry.value().addr() + offset)
    }

    pub fn set_flags(
        &mut self,
        address: VirtualAddress,
        flags: PageTableEntryFlags,
    ) -> Result<impl TableCacheFlush, ModifyMappingError> {
        let mut cache_flush = TableListCacheFlush::new();

        for walk_entry in self.walk_entries_mut(address) {
            let mut walk_entry = walk_entry?;
            let level = walk_entry.level();
            let entry = walk_entry.value_mut();

            if !entry.flags().contains(flags) {
                entry.set_flags(entry.flags() | flags);
                cache_flush.add_table(entry.as_frame(level));
            }
        }

        Ok(cache_flush)
    }

    fn borrow_owned_entries(&mut self) {
        let table = self.deref_l4_page_table_mut();

        let mut borrow_flag = PageTableEntryFlags::default();
        borrow_flag.set_borrowed(true);

        for entry in table.iter_mut() {
            if entry.flags().present() && !entry.flags().borrowed() {
                entry.set_flags(entry.flags() | borrow_flag);
            }
        }
    }

    pub fn new_mapper(&self, inherit: bool) -> Result<Self, NewMappingError> {
        let new_frame = self
            .frame_allocator
            .allocate_new_page_table()
            .ok_or(NewMappingError::OutOfFrames)?;

        let table = unsafe { self.deref_page_table_mut(new_frame.addr()) };

        if inherit {
            let clone_table = unsafe { self.deref_page_table_mut(self.l4_page.addr()) };

            for (i, clone_entry) in clone_table.iter().enumerate() {
                table[i] = if clone_entry.flags().borrowed() {
                    *clone_entry
                } else {
                    PageTableEntry::default()
                }
            }
        } else {
            table.zero();
        }

        Ok(Self {
            l4_page: new_frame,
            frame_allocator: self.frame_allocator,
            global_offset: self.global_offset,
        })
    }

    /// Create a new `MemoryMapper`.
    ///
    /// When the `inherit` parameter is set to `false`, the new `MemoryMapper` gets a
    /// empty address space.
    ///
    /// When `inherit` is set to `true` the memory map is shared between the current `MemoryMapper` and the new one.
    /// To avoid concurrency issues between these shared page tables, they cannot be modified by either the current and the new `MemoryMapper`, effectively being borrowed.
    /// Attempting to modify these tables anyways will result in a [`ModifyMappingError::NotOwned`] error.
    ///
    /// Be careful however when using this function, because the borrowed tables will **not** get deallocated when a `MemoryMapper` gets dropped, leaking memory ([which is safe btw](https://doc.rust-lang.org/std/boxed/struct.Box.html#method.leak)).
    /// This is because the solution to this problem is to reference count the borrowed tables, which is not acceptable for this performance-critical part of the kernel.
    /// However, this is not an issue when the `MemoryMapper` is used as described in _Use-case_ section on the struct-level documentation.
    /// Because the kernel mappings will of-course last for the entire execution time of the kernel.
    pub fn borrow_to_new_mapper(&mut self, inherit: bool) -> Result<Self, NewMappingError> {
        if inherit {
            self.borrow_owned_entries();
        }

        let new_frame = self
            .frame_allocator
            .allocate_new_page_table()
            .ok_or(NewMappingError::OutOfFrames)?;

        let table = unsafe { self.deref_page_table_mut(new_frame.addr()) };

        if inherit {
            let clone_table = unsafe { self.deref_page_table_mut(self.l4_page.addr()) };
            table.as_mut_slice().copy_from_slice(clone_table.as_slice());
        } else {
            table.zero();
        }

        Ok(Self {
            l4_page: new_frame,
            frame_allocator: self.frame_allocator,
            global_offset: self.global_offset,
        })
    }

    fn map_to_inner(
        &mut self,
        flags: PageTableEntryFlags,
        parent_flags: PageTableEntryFlags,
        new_page: VirtualPage,
        physical_address: PhysicalAddress,
    ) -> Result<impl TableCacheFlush, NewMappingError> {
        let mut cache_flush = TableListCacheFlush::new();
        let mut table_frame = self.l4_page;

        for (page_level, index) in Self::iter_address(new_page.addr()) {
            let table = unsafe { self.deref_page_table_mut(table_frame.addr()) };

            let mut entry = table[index];

            match (page_level, entry.flags().present()) {
                (1, true) => {
                    return Err(NewMappingError::AlreadyMapped);
                }
                (1, false) => {
                    entry.set_flags(flags);
                    entry.set_addr(physical_address);
                    table[index] = entry;
                    cache_flush.add_table(table_frame);
                }
                (_, false) => {
                    let allocated_page = self
                        .frame_allocator
                        .allocate_new_page_table()
                        .ok_or(NewMappingError::OutOfFrames)?;

                    let new_table = unsafe { self.deref_page_table_mut(allocated_page.addr()) };

                    new_table.zero();

                    entry.set_flags(parent_flags | entry.flags());
                    entry.set_addr(allocated_page.addr());
                    table[index] = entry;
                    cache_flush.add_table(table_frame);
                }
                _ => {}
            }

            table_frame = entry.as_frame(page_level);
        }

        Ok(cache_flush)
    }

    /// Creates a new mapping in the page table to the specified physical memory.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the you can create multiple mutable references to the same memory location, or trigger undefined behaviour to memory mapped IO when used incorrectly.
    pub unsafe fn map_to(
        &mut self,
        flags: PageTableEntryFlags,
        parent_flags: PageTableEntryFlags,
        new_page: VirtualPage,
        physical_address: PhysicalAddress,
    ) -> Result<impl TableCacheFlush, NewMappingError> {
        self.map_to_inner(flags, parent_flags, new_page, physical_address)
    }

    /// Creates a new mapping in the page table.
    pub fn new_map(
        &mut self,
        flags: PageTableEntryFlags,
        parent_flags: PageTableEntryFlags,
        new_page: VirtualPage,
    ) -> Result<impl TableCacheFlush, NewMappingError> {
        let frame = self
            .frame_allocator
            .allocate_new_page_table()
            .ok_or(NewMappingError::OutOfFrames)?;

        unsafe { self.map_to(flags, parent_flags, new_page, frame.addr()) }
    }

    fn deref_l4_page_table_mut(&mut self) -> &mut PageTable {
        // Safety: As stated in the constructor, the l4_page is guaranteed to point to valid data
        unsafe { self.deref_page_table_mut(self.l4_page.addr()) }
    }

    fn deref_l4_page_table(&self) -> &PageTable {
        // Safety: As stated in the constructor, the l4_page is guaranteed to point to valid data
        unsafe { self.deref_page_table(self.l4_page.addr()) }
    }

    /// Safety:
    /// The caller must ensure that the `addr` parameter points to a valid page table.
    unsafe fn deref_page_table(&self, addr: PhysicalAddress) -> &PageTable {
        let table_ptr: *const PageTable = self.translate_table_frame(addr).as_ptr();
        &*table_ptr
    }

    unsafe fn deref_page_table_mut(&self, addr: PhysicalAddress) -> &mut PageTable {
        let table_ptr: *mut PageTable = self.translate_table_frame(addr).as_mut_ptr();
        &mut *table_ptr
    }

    /// Set the memory map to the address space. In x86_64 terms, this means setting the CR3 register.
    pub fn set_active(&self) {
        unsafe { self.l4_page.make_active() }
    }

    /// Display the memory map as a tree view for debugging purposes.
    #[allow(dead_code)]
    pub fn tree_display(&self, max_depth: Option<u8>) -> impl Display + '_ {
        MemoryMapTreeDisplay::new(self, max_depth.unwrap_or(4))
    }

    fn walk_entries_mut(
        &mut self,
        address: VirtualAddress,
    ) -> impl Iterator<Item = Result<WalkEntry<&mut PageTableEntry>, WalkError>> + '_ {
        unsafe { MutPageWalker::new(self.l4_page.addr(), address, self) }
    }

    fn walk_entries(
        &self,
        address: VirtualAddress,
    ) -> impl Iterator<Item = Result<WalkEntry<&PageTableEntry>, WalkError>> + '_ {
        unsafe { PageWalker::new(address, self.l4_page.addr(), self) }
    }

    fn iter_address(address: VirtualAddress) -> impl Iterator<Item = (u8, usize)> {
        address
            .indices()
            .into_iter()
            .enumerate()
            .map(|(i, index)| (4 - i as u8, index as usize))
    }
}
