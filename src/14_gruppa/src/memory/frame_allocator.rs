use multiboot2::MemoryArea;
use crate::memory::{PhysicalAddress, PAGE_SIZE};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Frame {
    pub number: usize,
}

impl Frame {
    // Calculates the frame containing the given address.
    pub fn containing_address(address: usize) -> Self {
        Frame { number: address / PAGE_SIZE }
    }

    // Returns the starting physical address of the frame.
    pub fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }

    // Generates a range of frames from start to end.
    pub fn range_inclusive(start: Frame, end: Frame) -> impl Iterator<Item = Frame> {
        (start.number..=end.number).map(Frame::from_number)
    }

    // Helper to create a frame from a frame number.
    fn from_number(number: usize) -> Self {
        Frame { number }
    }
}

// Trait defining operations for frame allocation.
pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

// Allocator that manages frames based on memory areas.
pub struct AreaFrameAllocator {
    next_free_frame: Frame,
    memory_areas: &'static [MemoryArea],
    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
}

impl AreaFrameAllocator {
    // Initializes a new frame allocator with the specified kernel and multiboot memory areas.
    pub fn new(
        kernel_start: usize, kernel_end: usize,
        multiboot_start: usize, multiboot_end: usize,
        memory_areas: &'static [MemoryArea],
    ) -> Self {
        let mut allocator = AreaFrameAllocator {
            next_free_frame: Frame::containing_address(0),
            memory_areas,
            kernel_start: Frame::containing_address(kernel_start),
            kernel_end: Frame::containing_address(kernel_end),
            multiboot_start: Frame::containing_address(multiboot_start),
            multiboot_end: Frame::containing_address(multiboot_end),
        };
        allocator.choose_next_area(); // Chooses the first valid memory area to allocate frames from.
        allocator
    }

    // Selects the next usable memory area that has not been allocated yet.
    fn choose_next_area(&mut self) {
        let start_frame = &self.next_free_frame;
        self.next_free_frame = self.memory_areas.iter()
            .filter_map(|area| {
                let end_address = area.start_address() as usize + area.size() as usize - 1;
                let end_frame = Frame::containing_address(end_address);
                if end_frame >= *start_frame { Some(Frame::containing_address(area.start_address() as usize)) } else { None }
            })
            .min().unwrap_or(start_frame.clone());
    }
}

impl FrameAllocator for AreaFrameAllocator {
    // Allocates the next free frame if available.
    fn allocate_frame(&mut self) -> Option<Frame> {
        while let Some(frame) = (self.next_free_frame.number..).find_map(|n| {
            let frame = Frame::from_number(n);
            // Check if the frame is outside the reserved kernel and multiboot areas
            if (frame < self.kernel_start || frame > self.kernel_end) &&
               (frame < self.multiboot_start || frame > self.multiboot_end) {
                Some(frame)
            } else {
                None
            }
        }) {
            // Move to the next frame for future allocations
            self.next_free_frame = Frame::from_number(frame.number + 1);
            return Some(frame);
        }
        None
    }

    fn deallocate_frame(&mut self, _frame: Frame) {
        unimplemented!()
    }
}
