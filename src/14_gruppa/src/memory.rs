use x86_64::{PhysAddr, VirtAddr};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::structures::paging::{PageTable, PhysFrame, FrameAllocator, OffsetPageTable, Size4KiB};

pub unsafe fn initialize_offset_page_table(physical_offset: VirtAddr) -> OffsetPageTable<'static> {
    OffsetPageTable::new(get_active_lvl4_table(physical_offset), physical_offset)
}

pub struct MemoryFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl MemoryFrameAllocator {
    pub unsafe fn new(memory_map: &'static MemoryMap) -> Self {
        Self { memory_map, next: 0 }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> + '_ {
        self.memory_map.iter()
            .filter(|region| region.region_type == MemoryRegionType::Usable)
            .flat_map(|region| region.range.start_addr()..region.range.end_addr())
            .step_by(4096)
            .map(|address| PhysFrame::containing_address(PhysAddr::new(address)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for MemoryFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += frame.is_some() as usize;
        frame
    }
}

pub unsafe fn get_active_lvl4_table(physical_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_offset + phys.as_u64();
    &mut *(virt.as_mut_ptr())
}
