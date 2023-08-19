use crate::arch::x86_64::paging::PhysicalPage;
use crate::util::collections::FixedVec;

pub trait TableCacheFlush {
    fn flush(self);
}

pub struct TableListCacheFlush {
    tables: FixedVec<4, PhysicalPage>
}

impl TableListCacheFlush {
    pub const fn new() -> Self {
        Self {
            tables: FixedVec::new()
        }
    }

    pub fn add_table(&mut self, table: PhysicalPage) {
        self.tables.push(table);
    }
}

impl TableCacheFlush for TableListCacheFlush {
    fn flush(self) {
        for table in self.tables.iter() {
            table.flush();
        }
    }
}