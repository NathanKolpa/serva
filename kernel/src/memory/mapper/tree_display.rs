use core::fmt::{write, Display, Formatter};

use crate::arch::x86_64::paging::{PageTable, PageTableEntryFlags};
use crate::memory::MemoryMapper;
use crate::util::collections::FixedVec;

pub struct MemoryMapTreeDisplay<'a> {
    memory_mapper: &'a MemoryMapper,
    max_depth: u8,
}

impl<'a> MemoryMapTreeDisplay<'a> {
    pub fn new(memory_mapper: &'a MemoryMapper, max_depth: u8) -> Self {
        assert!(max_depth <= 4);
        Self {
            memory_mapper,
            max_depth,
        }
    }

    fn print_indentation_line(
        f: &mut Formatter<'_>,
        depth: u8,
        skip: &[bool],
    ) -> core::fmt::Result {
        for i in 0..depth {
            let char = if skip[i as usize] { " " } else { "│" };

            write!(f, "{char}   ")?;
        }

        Ok(())
    }

    fn print_present_indentation_line(
        f: &mut Formatter<'_>,
        depth: u8,
        skip: &[bool],
        last: bool,
    ) -> core::fmt::Result {
        Self::print_indentation_line(f, depth, skip)?;

        if last {
            write!(f, "└")?;
        } else {
            write!(f, "├")?;
        }

        write!(f, "───")
    }

    fn print_skipped_indentation_line(
        f: &mut Formatter<'_>,
        depth: u8,
        skip: &[bool],
        start: usize,
        end: usize,
    ) -> core::fmt::Result {
        Self::print_indentation_line(f, depth, skip)?;
        write!(f, "{start}..{end}")
    }

    fn print_page_table(
        &self,
        f: &mut Formatter<'_>,
        depth: u8,
        skip_list: &mut [bool],
        table: &PageTable,
    ) -> core::fmt::Result {
        let mut skips: usize = 0;
        let last_element = table
            .iter()
            .enumerate()
            .rfind(|(_, x)| x.flags().present())
            .map(|x| x.0);

        for (i, entry) in table.iter().enumerate() {
            let present = entry.flags().present();

            if present {
                let is_last = Some(i) == last_element;

                if skips == 1 {
                    Self::print_indentation_line(f, depth, skip_list)?;
                    writeln!(f, "│")?;
                } else if skips > 0 {
                    Self::print_skipped_indentation_line(f, depth, skip_list, i - skips, i - 1)?;
                    writeln!(f)?;
                }

                Self::print_present_indentation_line(f, depth, skip_list, is_last)?;

                writeln!(f, "{i}: [{entry}]")?;

                if !entry.flags().huge() && depth < self.max_depth {
                    if is_last {
                        skip_list[depth as usize] = true;
                    }

                    let table = unsafe { self.memory_mapper.deref_page_table(entry.addr()) };
                    self.print_page_table(f, depth + 1, skip_list, table)?;

                    skip_list[depth as usize] = false;
                }

                skips = 0;
            } else {
                skips += 1;
            }
        }

        Ok(())
    }
}

impl Display for MemoryMapTreeDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut skip_list = FixedVec::<4, _>::initialized_with(false);
        writeln!(f, "CR3")?;
        self.print_page_table(
            f,
            0,
            &mut skip_list,
            self.memory_mapper.deref_l4_page_table(),
        )
    }
}
