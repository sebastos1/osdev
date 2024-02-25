pub const PAGE_SIZE: usize = 4096;
use multiboot2::MemoryArea;
use linked_list_allocator::LockedHeap;

// temp
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    number: usize,
}
impl Frame {
    fn containing_address(address: usize) -> Frame {
        Frame{ number: address / PAGE_SIZE }
    }
}
pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}


pub struct AreaFrameAllocator<'a> {
    next_free_frame: Frame,
    current_area: Option<&'a MemoryArea>,
    memory_areas: &'a [MemoryArea],
    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
}

impl AreaFrameAllocator<'_> {
    pub fn new(
        kernel_start: usize, 
        kernel_end: usize,
        multiboot_start: usize, 
        multiboot_end: usize,
        memory_areas: &[MemoryArea]
    ) -> AreaFrameAllocator {
        let mut allocator = AreaFrameAllocator {
            next_free_frame: Frame::containing_address(0),
            current_area: None,
            memory_areas: memory_areas,
            kernel_start: Frame::containing_address(kernel_start),
            kernel_end: Frame::containing_address(kernel_end),
            multiboot_start: Frame::containing_address(multiboot_start),
            multiboot_end: Frame::containing_address(multiboot_end),
        };
        allocator.choose_next_area();
        allocator
    }

    fn choose_next_area(&mut self) {
        self.current_area = self.memory_areas.iter().filter(|area| {
            let address = area.start_address() + area.size() - 1;
            Frame::containing_address(address as usize) >= self.next_free_frame
        }).min_by_key(|area| area.start_address());

        if let Some(area) = self.current_area {
            let start_frame = Frame::containing_address(area.start_address() as usize);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}

impl FrameAllocator for AreaFrameAllocator<'_> {
    fn allocate_frame(&mut self) -> Option<Frame> {
        while let Some(area) = self.current_area {
            let frame = Frame { number: self.next_free_frame.number };

            let current_area_last_frame = Frame::containing_address(area.start_address() as usize + area.size() as usize - 1);

            if frame > current_area_last_frame {
                self.choose_next_area();
                continue;
            }

            if frame >= self.kernel_start && frame <= self.kernel_end || frame >= self.multiboot_start && frame <= self.multiboot_end {
                self.next_free_frame.number += 1; 
                continue;
            }

            self.next_free_frame.number += 1;
            return Some(frame);
        }

        None
    }
    
    fn deallocate_frame(&mut self, _frame: Frame) { unimplemented!() }
}