use core::ops::Deref;
use multiboot2::ElfSection;
use crate::memory::paging::{
    EntryFlags,
    mapper::Mapper,
    temporary_page::TemporaryPage,
    page::{ActivePageTable, InactivePageTable, Page, PageIter},
};
use frame_allocator::{Frame, FrameAllocator};

mod frame_allocator;
mod heap_allocator;
pub mod paging;
mod stack_allocator;

const ENTRY_COUNT: usize = 512;

pub const HEAP_START: usize = 0o_000_001_000_000_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB
pub const PAGE_SIZE: usize = 4096;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub fn init() -> MemoryController {
    let boot_info = crate::BOOT_INFO
        .wait()
        .expect("BootInformation not initialized");

    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");
    let elf_sections = boot_info.elf_sections().expect("elf sections required");
    let kernel_start = elf_sections
        .clone()
        .map(|s| s.start_address())
        .min()
        .unwrap();
    let kernel_end = elf_sections
        .map(|s| s.start_address() + s.size())
        .max()
        .unwrap();

    let mut frame_allocator = frame_allocator::AreaFrameAllocator::new(
        kernel_start as usize,
        kernel_end as usize,
        boot_info.start_address(),
        boot_info.end_address(),
        memory_map_tag.memory_areas(),
    );

    let mut active_table = self::paging::remap_the_kernel(&mut frame_allocator, &boot_info);

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
    active_table: ActivePageTable,
    frame_allocator: self::frame_allocator::AreaFrameAllocator,
    stack_allocator: stack_allocator::StackAllocator,
}

impl MemoryController {
    pub fn alloc_stack(&mut self, size_in_pages: usize) -> Option<Stack> {
        let &mut MemoryController {
            ref mut active_table,
            ref mut frame_allocator,
            ref mut stack_allocator,
        } = self;
        stack_allocator.alloc_stack(active_table, frame_allocator, size_in_pages)
    }
}
