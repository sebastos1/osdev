use multiboot2::MemoryArea;
use multiboot2::BootInformation;

pub const PAGE_SIZE: usize = 4096; // 4 KiB

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct Frame {
    pub number: usize,
}

impl Frame {
    fn with_address(address: usize) -> Self {
        Frame { number: address / PAGE_SIZE }
    }

    fn from_number(number: usize) -> Self {
        Frame { number }
    }
}

pub struct FrameAllocator {
    pub next_free: Frame,
    memory_areas: &'static [MemoryArea],
    kernel_start: Frame,
    kernel_end: Frame,
    bootinfo_start: Frame,
    bootinfo_end: Frame,
}

impl FrameAllocator {
    pub fn new(
        boot_info: &'static BootInformation
    ) -> Self {
        let memory_map_tag = boot_info.memory_map_tag().unwrap();

        let bootinfo_start = boot_info.start_address();
        let bootinfo_end = boot_info.end_address();

        let elf_sections = boot_info.elf_sections().unwrap();
        let kernel_start = elf_sections.clone().map(|s| s.start_address()).min().unwrap();
        let kernel_end = elf_sections.map(|s| s.start_address() + s.size()).max().unwrap();

        let mut allocator = FrameAllocator {
            next_free: Frame::with_address(0),
            memory_areas: memory_map_tag.memory_areas(),
            kernel_start: Frame::with_address(kernel_start as usize),
            kernel_end: Frame::with_address(kernel_end as usize),
            bootinfo_start: Frame::with_address(bootinfo_start),
            bootinfo_end: Frame::with_address(bootinfo_end),
        };
        allocator.choose_area();
        allocator
    }

    fn choose_area(&mut self) {
        let start_frame = &self.next_free;
        self.next_free = self.memory_areas.iter()
            .filter_map(|area| {
                let end_address = area.start_address() as usize + area.size() as usize - 1;
                let end_frame = Frame::with_address(end_address);
                if end_frame >= *start_frame { Some(Frame::with_address(area.start_address() as usize)) } else { None }
            })
            .min().unwrap_or(start_frame.clone());
    }

    pub fn allocate(&mut self) -> Option<Frame> {
        while let Some(frame) = (self.next_free.number..).find_map(|n| {
            let frame = Frame::from_number(n);
            if (frame < self.kernel_start || frame > self.kernel_end) &&
               (frame < self.bootinfo_start || frame > self.bootinfo_end) {
                Some(frame)
            } else {
                None
            }
        }) {
            self.next_free = Frame::from_number(frame.number + 1);
            return Some(frame);
        }
        None
    }

    fn deallocate(&mut self, _frame: Frame) {
        unimplemented!();
    }
}