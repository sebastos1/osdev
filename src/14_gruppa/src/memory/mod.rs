use core::arch::asm;
pub use self::entry::*;
pub use self::mapper::Mapper;
use multiboot2::BootInformation;
use core::ops::{Deref, DerefMut};
use self::temporary_page::TemporaryPage;
use frame_allocator::{Frame, FrameAllocator};

mod entry;
mod table;
mod mapper;
mod temporary_page;
pub mod heap_allocator;
pub mod frame_allocator;

const ENTRY_COUNT: usize = 512;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub const HEAP_START: usize = 0o_000_001_000_000_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB
pub const PAGE_SIZE: usize = 4096;

pub fn init(multiboot_information_address: usize) {
    let boot_info = unsafe { 
        multiboot2::BootInformation::load(
            multiboot_information_address as *const multiboot2::BootInformationHeader
        ).unwrap() 
    };
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");
    let elf_sections = boot_info.elf_sections().expect("elf sections required");
    let kernel_start = elf_sections.clone().map(|s| s.start_address()).min().unwrap();
    let kernel_end = elf_sections.map(|s| s.start_address() + s.size()).max().unwrap();
 
    let mut frame_allocator = frame_allocator::AreaFrameAllocator::new(
        kernel_start as usize, 
        kernel_end as usize, 
        boot_info.start_address(),
        boot_info.end_address(), 
        memory_map_tag.memory_areas()
    );

    let mut active_table = remap_the_kernel(&mut frame_allocator, &boot_info);

    let heap_start_page = Page::containing_address(HEAP_START);
    let heap_end_page = Page::containing_address(HEAP_START + HEAP_SIZE - 1);

    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        active_table.map(page, entry::EntryFlags::WRITABLE, &mut frame_allocator);
    }


    // unsafe {
    //     HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_START + HEAP_SIZE);
    // }
    // use crate::paging::Page;    

    // let heap_start_page = Page::containing_address(HEAP_START);
    // let heap_end_page = Page::containing_address(HEAP_START + HEAP_SIZE-1);

    // for page in Page::range_inclusive(heap_start_page, heap_end_page) {
    //     active_table.map(page, paging::EntryFlags::WRITABLE, &mut frame_allocator);
    // }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
   number: usize,
}

impl Page {
    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(address < 0x0000_8000_0000_0000 ||
            address >= 0xffff_8000_0000_0000,
            "invalid address: 0x{:x}", address);
        Page { number: address / PAGE_SIZE }
    }

    fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }

    fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }
    fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }
    fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }
    fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }

    pub fn range_inclusive(start: Page, end: Page) -> PageIter {
        PageIter {
            start: start,
            end: end,
        }
    }
}

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
    unsafe fn new() -> ActivePageTable {
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
    p4_frame: Frame,
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

pub fn remap_the_kernel<A>(allocator: &mut A, boot_info: &BootInformation) -> ActivePageTable
    where A: FrameAllocator
{
    let mut temporary_page = TemporaryPage::new(Page { number: 0xcafebabe }, allocator);

    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = {
        let frame = allocator.allocate_frame().expect("no more frames");
        InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
    };

    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
        let elf_sections_tag = boot_info.elf_sections().expect("Memory map tag required");

        for section in elf_sections_tag {
            if !section.is_allocated() { continue; }

            assert!(section.start_address() % PAGE_SIZE as u64 == 0, "sections need to be page aligned");

            println!("mapping section at addr: {:#x}, size: {:#x}", section.start_address(), section.size());

            let flags = EntryFlags::from_elf_section_flags(&section);

            let start_frame = Frame::containing_address(section.start_address() as usize);

            let end_frame = Frame::containing_address((section.end_address() - 1) as usize);

            for frame in Frame::range_inclusive(start_frame, end_frame) {
                mapper.identity_map(frame, flags.clone(), allocator);
            }
        }

        // identity map the VGA text buffer
        let vga_buffer_frame = Frame::containing_address(0xb8000);

        mapper.identity_map(vga_buffer_frame, EntryFlags::WRITABLE.clone(), allocator);

        // identity map the multiboot info structure
        let multiboot_start = Frame::containing_address(boot_info.start_address());
        let multiboot_end = Frame::containing_address(boot_info.end_address() - 1);
        for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
            mapper.identity_map(frame, EntryFlags::PRESENT.clone(), allocator);
        }
    });

    let old_table = active_table.switch(new_table);

    // turn the old p4 page into a guard page
    let old_p4_page = Page::containing_address(
      old_table.p4_frame.start_address()
    );
    active_table.unmap(old_p4_page, allocator);
    println!("guard page at {:#x}", old_p4_page.start_address());

    active_table
}