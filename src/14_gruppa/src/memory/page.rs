use crate::memory::EntryFlags;
use crate::memory::Mapper;
use crate::memory::Deref;
use crate::memory::PAGE_SIZE;
use crate::memory::Frame;
use crate::memory::TemporaryPage;
use crate::memory::temporary_page;
use core::arch::asm;
use crate::memory::VirtualAddress;
use core::ops::DerefMut;
use core::ops::Add;

// page, pageiter, active/inactive pagetable

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    pub number: usize,
}

impl Page {
    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_0000, "invalid address: 0x{:x}", address);
        Page { number: address / PAGE_SIZE }
    }

    pub fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }

    pub fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }
    pub fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }
    pub fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }
    pub fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }

    pub fn range_inclusive(start: Page, end: Page) -> PageIter {
        PageIter {
            start: start,
            end: end,
        }
    }
}
impl Add<usize> for Page {
    type Output = Page;

    fn add(self, rhs: usize) -> Page {
        Page { number: self.number + rhs }
    }
}


#[derive(Clone)]
pub struct PageIter {
    start: Page,
    end: Page,
}

impl Iterator for PageIter {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
        if self.start <= self.end {
            let page = self.start;
            self.start.number += 1;
            Some(page)
        } else {
            None
        }
    }
}

pub struct ActivePageTable {
    mapper: Mapper,
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

impl ActivePageTable {
    pub unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            mapper: Mapper::new(),
        }
    }

    pub fn with<F>(
        &mut self,
        table: &mut InactivePageTable,
        temporary_page: &mut temporary_page::TemporaryPage,
        f: F
    ) where F: FnOnce(&mut Mapper) 
    {
        use x86_64::instructions::tlb;

        {
            let cr3_value: usize;
            unsafe { asm!("mov {}, cr3", out(reg) cr3_value); } // read Cr3 register
            let backup = Frame::containing_address(cr3_value);

            // map temporary_page to current p4 table
            let p4_table = temporary_page.map_table_frame(backup.clone(), self);

            // overwrite recursive mapping
            self.p4_mut()[511].set(table.p4_frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
            tlb::flush_all();

            // execute f in the new context
            f(self);

            // restore recursive mapping to original p4 table
            p4_table[511].set(backup, EntryFlags::PRESENT | EntryFlags::WRITABLE);
            tlb::flush_all();
        }

        temporary_page.unmap(self);
    }

    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {

        let cr3_value: usize;
        unsafe { asm!("mov {}, cr3", out(reg) cr3_value); }

        let old_table = InactivePageTable {
            p4_frame: Frame::containing_address(cr3_value),
        };
        let new_cr3_value = new_table.p4_frame.start_address();
        unsafe {
            asm!("mov cr3, {}", in(reg) new_cr3_value, options(nostack));
        }
        old_table
    }
}

pub struct InactivePageTable {
    pub p4_frame: Frame,
}
impl InactivePageTable {
    pub fn new(
            frame: Frame,
            active_table: &mut ActivePageTable,
            temporary_page: &mut TemporaryPage
    ) -> InactivePageTable {
        let table = temporary_page.map_table_frame(frame.clone(), active_table);
        table.zero();
        table[511].set(frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
        temporary_page.unmap(active_table);

        InactivePageTable { p4_frame: frame }
    }
}