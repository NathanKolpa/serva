use crate::arch::x86_64::segmentation::segment::*;
use crate::arch::x86_64::segmentation::selector::SegmentSelector;
use crate::arch::x86_64::privilege::PrivilegeLevel;
use crate::arch::x86_64::tables::DescriptorTablePointer;
use crate::util::address::VirtualAddress;
use core::mem::size_of;

pub enum SegmentDescriptor {
    NormalSegment(NormalSegment<UserAccessByte>),
    NormalSystemSegment(NormalSegment<SystemAccessByte>),
    LongSystemSegment(LongSegment),
}

impl SegmentDescriptor {
    pub fn privilege(&self) -> PrivilegeLevel {
        match self {
            SegmentDescriptor::NormalSegment(s) => s.privilege(),
            SegmentDescriptor::NormalSystemSegment(s) => s.privilege(),
            SegmentDescriptor::LongSystemSegment(s) => s.privilege(),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            SegmentDescriptor::NormalSegment(_) | SegmentDescriptor::NormalSystemSegment(_) => 1,
            SegmentDescriptor::LongSystemSegment(_) => 2,
        }
    }
}

pub struct GlobalDescriptorTable {
    table: [u64; 8],
    len: usize,
}

impl GlobalDescriptorTable {
    pub const fn new() -> Self {
        Self {
            table: [NormalSegment::NULL.as_u64(); 8],
            len: 1,
        }
    }

    pub fn add_entry(&mut self, descriptor: SegmentDescriptor) -> Option<SegmentSelector> {
        let index = self.len;
        let descriptor_size = descriptor.size();

        if index + descriptor_size > self.table.len() {
            return None;
        }

        self.len += descriptor_size;

        match descriptor {
            SegmentDescriptor::NormalSegment(segment) => {
                self.table[index] = segment.as_u64();
            }
            SegmentDescriptor::NormalSystemSegment(segment) => {
                self.table[index] = segment.as_u64();
            }
            SegmentDescriptor::LongSystemSegment(segment) => {
                let (lower, higher) = segment.as_u64();
                self.table[index] = lower;
                self.table[index + 1] = higher;
            }
        }

        Some(SegmentSelector::new(index as u16, descriptor.privilege()))
    }

    pub fn load(&'static self) {
        let pointer = self.pointer();

        unsafe {
            pointer.load_descriptor_table();
        }
    }

    fn pointer(&self) -> DescriptorTablePointer {
        DescriptorTablePointer {
            base: VirtualAddress::new(self as *const _ as u64),
            limit: (self.len * size_of::<u64>() - 1) as u16,
        }
    }
}
