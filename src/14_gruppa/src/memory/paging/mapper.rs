use core::ptr::NonNull;
use super::{EntryFlags, table::{self, Level4, Table}};
use crate::memory::{Frame, FrameAllocator, Page, PhysicalAddress, VirtualAddress, PAGE_SIZE};

// Handles page table translations and modifications.
pub struct Mapper {
    p4: NonNull<Table<Level4>>,
}

impl Mapper {
    // Creates a new Mapper. Unsafe because it assumes the physical memory address of P4 is valid.
    pub unsafe fn new() -> Mapper {
        Mapper {
            p4: NonNull::new_unchecked(table::P4),
        }
    }

    // Accessors for the P4 table, safely returning references to the P4 table.
    pub fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.as_ref() }
    }

    pub fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.as_mut() }
    }

    // Translates a virtual address to a physical address, if possible.
    #[allow(unused)]
    pub fn translate(&self, virtual_address: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = virtual_address % PAGE_SIZE;
        self.translate_page(Page::containing_address(virtual_address)).map(|frame| frame.start_address() + offset)
    }

    // Helper function to handle page translation, supporting huge pages.
    pub fn translate_page(&self, page: Page) -> Option<Frame> {
        self.p4().next_table(page.p4_index())
            .and_then(|p3| p3.next_table(page.p3_index()))
            .and_then(|p2| p2.next_table(page.p2_index()))
            .and_then(|p1| p1[page.p1_index()].pointed_frame())
            .or_else(|| self.handle_huge_pages(page))
    }

    // Maps a page to a frame with the specified flags, allocating new page tables as necessary.
    pub fn map_to<A>(&mut self, page: Page, frame: Frame, flags: EntryFlags, allocator: &mut A)
    where A: FrameAllocator {
        let p1 = self.navigate_to_p1_table(page, allocator);
        assert!(p1[page.p1_index()].is_unused(), "Page already mapped");
        p1[page.p1_index()].set(frame, flags | EntryFlags::PRESENT);
    }

    // High-level mapping function that allocates a frame and maps a page to it.
    pub fn map<A>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A)
    where A: FrameAllocator {
        let frame = allocator.allocate_frame().expect("Out of memory");
        self.map_to(page, frame, flags, allocator);
    }

    // Maps a frame to the corresponding page with the same address (identity mapping).
    pub fn identity_map<A>(&mut self, frame: Frame, flags: EntryFlags, allocator: &mut A)
    where A: FrameAllocator {
        self.map_to(Page::containing_address(frame.start_address()), frame, flags, allocator);
    }

    // Unmaps a page and flushes the corresponding TLB entry.
    pub fn unmap<A>(&mut self, page: Page, _allocator: &mut A)
    where A: FrameAllocator {
        use x86_64::instructions::tlb;
        use x86_64::VirtAddr;

        let p1 = self.navigate_to_p1_table_mut(page).expect("Mapping code does not support huge pages");
        p1[page.p1_index()].set_unused();
        tlb::flush(VirtAddr::new(page.start_address() as u64));
    }

    // Navigates through the page tables to the P1 table, creating tables if necessary.
    fn navigate_to_p1_table<A>(&mut self, page: Page, allocator: &mut A) -> &mut Table<table::Level1>
    where A: FrameAllocator {
        let p3 = self.p4_mut().next_table_create(page.p4_index(), allocator);
        let p2 = p3.next_table_create(page.p3_index(), allocator);
        p2.next_table_create(page.p2_index(), allocator)
    }

    // Similar to `navigate_to_p1_table` but does not attempt to create missing tables.
    fn navigate_to_p1_table_mut(&mut self, page: Page) -> Option<&mut Table<table::Level1>> {
        self.p4_mut().next_table_mut(page.p4_index())?
            .next_table_mut(page.p3_index())?
            .next_table_mut(page.p2_index())
    }

    // todo, someday
    #[allow(unused)]
    fn handle_huge_pages(&self, page: Page) -> Option<Frame> {
        None
    }
}
