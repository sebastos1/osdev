use crate::memory::{
    ActivePageTable, Frame, FrameAllocator, InactivePageTable, Page, TemporaryPage, PAGE_SIZE,
};
use multiboot2::{BootInformation, ElfSection};

pub mod entry;
pub mod mapper;
pub mod page;
pub mod table;
pub mod temporary_page;

// Defines flags used in page table entries with specific meanings.
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

// Converts ELF section flags to page table entry flags.
impl EntryFlags {
    pub fn from_elf_section_flags(section: &ElfSection) -> EntryFlags {
        use multiboot2::ElfSectionFlags;
        let mut flags = EntryFlags::empty();

        if section.flags().contains(ElfSectionFlags::ALLOCATED) {
            flags |= EntryFlags::PRESENT;
        }
        if section.flags().contains(ElfSectionFlags::WRITABLE) {
            flags |= EntryFlags::WRITABLE;
        }
        if !section.flags().contains(ElfSectionFlags::EXECUTABLE) {
            flags |= EntryFlags::NO_EXECUTE;
        }
        flags
    }
}

// Remaps the kernel using a new page table to better manage memory.
pub fn remap_the_kernel<A>(allocator: &mut A, boot_info: &BootInformation) -> ActivePageTable
where
    A: FrameAllocator,
{
    let mut temporary_page = TemporaryPage::new(Page { number: 0xcafebabe }, allocator);
    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = InactivePageTable::new(
        allocator.allocate_frame().expect("no more frames"),
        &mut active_table,
        &mut temporary_page,
    );

    // Setup new mappings within the new table.
    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
        // Map ELF sections
        let elf_sections_tag = boot_info.elf_sections().expect("Memory map tag required");
        for section in elf_sections_tag {
            if !section.is_allocated() { continue; }
            assert!(section.start_address() % PAGE_SIZE as u64 == 0, "Sections need to be page aligned");
            
            let flags = EntryFlags::from_elf_section_flags(&section);
            let start_frame = Frame::containing_address(section.start_address() as usize);
            let end_frame = Frame::containing_address((section.end_address() - 1) as usize);
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                mapper.identity_map(frame, flags.clone(), allocator);
            }
        }

        // Map VGA text buffer
        mapper.identity_map(Frame::containing_address(0xb8000), EntryFlags::WRITABLE, allocator);

        // Map Multiboot structure
        let multiboot_start = Frame::containing_address(boot_info.start_address());
        let multiboot_end = Frame::containing_address(boot_info.end_address() - 1);
        for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
            mapper.identity_map(frame, EntryFlags::PRESENT, allocator);
        }
    });

    // Switch to the new table and clean up the old one
    let old_table = active_table.switch(new_table);
    active_table.unmap(Page::containing_address(old_table.p4_frame.start_address()), allocator);

    active_table
}
