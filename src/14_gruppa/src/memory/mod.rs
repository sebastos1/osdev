use core::arch::asm;
pub use self::entry::*;
use multiboot2::ElfSection;
pub use self::mapper::Mapper;
use multiboot2::BootInformation;
use core::ops::{Deref, DerefMut};
use self::temporary_page::TemporaryPage;
use frame_allocator::{Frame, FrameAllocator};
use crate::memory::page::{Page, PageIter, ActivePageTable};
use crate::memory::page::InactivePageTable;

mod entry;
pub mod frame_allocator;
pub mod heap_allocator;
mod mapper;
mod page;
mod stack_allocator;
mod table;
mod temporary_page;

const ENTRY_COUNT: usize = 512;

pub const HEAP_START: usize = 0o_000_001_000_000_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB
pub const PAGE_SIZE: usize = 4096;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub fn init() -> MemoryController {
    let boot_info = crate::BOOT_INFO.wait().expect("BootInformation not initialized");

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
        active_table.map(page, EntryFlags::WRITABLE, &mut frame_allocator);
    }

    let stack_allocator = {
        let stack_alloc_start = heap_end_page + 1;
        let stack_alloc_end = stack_alloc_start + 100;
        let stack_alloc_range = Page::range_inclusive(stack_alloc_start, stack_alloc_end);
        stack_allocator::StackAllocator::new(stack_alloc_range)
    };

    MemoryController {
        active_table: active_table,
        frame_allocator: frame_allocator,
        stack_allocator: stack_allocator,
    }
}

// move this somewhere neater
pub use self::stack_allocator::Stack;

pub struct MemoryController {
    active_table: self::page::ActivePageTable,
    frame_allocator: self::frame_allocator::AreaFrameAllocator,
    stack_allocator: stack_allocator::StackAllocator,
}

impl MemoryController {
    pub fn alloc_stack(&mut self, size_in_pages: usize) -> Option<Stack> {
        let &mut MemoryController { 
            ref mut active_table,
            ref mut frame_allocator,
            ref mut stack_allocator
        } = self;
        stack_allocator.alloc_stack(active_table, frame_allocator, size_in_pages)
    }
}


bitflags! {
    #[derive(Clone)]
    pub struct EntryFlags: u64 {
        const PRESENT =         1 << 0;
        const WRITABLE =        1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH =   1 << 3;
        const NO_CACHE =        1 << 4;
        const ACCESSED =        1 << 5;
        const DIRTY =           1 << 6;
        const HUGE_PAGE =       1 << 7;
        const GLOBAL =          1 << 8;
        const NO_EXECUTE =      1 << 63;
    }
}
impl EntryFlags {
    pub fn from_elf_section_flags(section: &ElfSection) -> EntryFlags {
        use multiboot2::ElfSectionFlags;
        let mut flags = EntryFlags::empty();

        if section.flags().contains(ElfSectionFlags::ALLOCATED) {
            flags = flags | EntryFlags::PRESENT;
        }
        if section.flags().contains(ElfSectionFlags::WRITABLE) {
            flags = flags | EntryFlags::WRITABLE;
        }
        if !section.flags().contains(ElfSectionFlags::EXECUTABLE) {
            flags = flags | EntryFlags::NO_EXECUTE;
        }
        flags
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