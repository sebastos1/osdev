use core::arch::asm;
use core::ops::{Add, Deref, DerefMut};
use crate::memory::{Frame, Mapper, PAGE_SIZE, VirtualAddress};
use super::{EntryFlags, temporary_page::TemporaryPage};

// Represents a virtual page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    pub number: usize,
}

impl Page {
    // Creates a page containing the given address, ensuring it's in a valid range.
    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_0000, "invalid address: 0x{:x}", address);
        Page { number: address / PAGE_SIZE }
    }

    // Returns the start address of the page.
    pub fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }

    // Index functions return the respective index for each table level.
    pub fn p4_index(&self) -> usize { (self.number >> 27) & 0o777 }
    pub fn p3_index(&self) -> usize { (self.number >> 18) & 0o777 }
    pub fn p2_index(&self) -> usize { (self.number >> 9) & 0o777 }
    pub fn p1_index(&self) -> usize { self.number & 0o777 }

    // Creates an iterator for a range of pages.
    pub fn range_inclusive(start: Page, end: Page) -> PageIter {
        PageIter { start, end }
    }
}

// Allows adding a number of pages to a Page, returning a new Page.
impl Add<usize> for Page {
    type Output = Page;
    fn add(self, rhs: usize) -> Page {
        Page { number: self.number + rhs }
    }
}

// Iterator for a range of pages.
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

// Represents the current active page table.
pub struct ActivePageTable {
    mapper: Mapper,
}

impl Deref for ActivePageTable {
    type Target = Mapper;
    fn deref(&self) -> &Mapper { &self.mapper }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper { &mut self.mapper }
}

impl ActivePageTable {
    // Creates a new ActivePageTable. Unsafe because it directly manipulates hardware.
    pub unsafe fn new() -> ActivePageTable {
        ActivePageTable { mapper: Mapper::new() }
    }

    // Temporarily maps an inactive page table to perform an operation.
    pub fn with<F>(&mut self, table: &mut InactivePageTable, temporary_page: &mut TemporaryPage, f: F)
    where F: FnOnce(&mut Mapper) {
        let backup = Frame::containing_address(unsafe { ActivePageTable::read_cr3() });
        let p4_table = temporary_page.map_table_frame(backup.clone(), self);

        // Overwrite recursive mapping.
        self.p4_mut()[511].set(table.p4_frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
        Self::flush_tlb();

        f(self); // Execute provided function in the context of the new table.

        // Restore original mapping.
        p4_table[511].set(backup, EntryFlags::PRESENT | EntryFlags::WRITABLE);
        Self::flush_tlb();

        temporary_page.unmap(self);
    }

    // Switches to a new page table, returning the old one.
    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        let old_table = InactivePageTable { p4_frame: Frame::containing_address(unsafe { ActivePageTable::read_cr3() }) };
        unsafe { ActivePageTable::write_cr3(new_table.p4_frame.start_address()); }
        old_table
    }

    // Reads the current value of the CR3 register.
    unsafe fn read_cr3() -> usize {
        let cr3_value: usize;
        asm!("mov {}, cr3", out(reg) cr3_value);
        cr3_value
    }

    // Writes a new value to the CR3 register, changing the active page table.
    unsafe fn write_cr3(value: usize) {
        asm!("mov cr3, {}", in(reg) value, options(nostack));
    }

    // Flushes the Translation Lookaside Buffer (TLB).
    fn flush_tlb() {
        use x86_64::instructions::tlb;
        tlb::flush_all();
    }
}

// Represents an inactive page table.
pub struct InactivePageTable {
    pub p4_frame: Frame,
}

impl InactivePageTable {
    // Creates a new InactivePageTable, mapping the provided frame as the P4 table and zeroing it.
    pub fn new(frame: Frame, active_table: &mut ActivePageTable, temporary_page: &mut TemporaryPage) -> InactivePageTable {
        let table = temporary_page.map_table_frame(frame.clone(), active_table);
        table.zero();
        table[511].set(frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
        temporary_page.unmap(active_table);
        InactivePageTable { p4_frame: frame }
    }
}
