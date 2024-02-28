use multiboot2::MemoryArea;
use crate::memory::{PhysicalAddress, PAGE_SIZE};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    pub number: usize,
}
impl Frame {
    pub fn containing_address(address: usize) -> Frame {
        Frame {
            number: address / PAGE_SIZE,
        }
    }

    pub fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }

    pub fn clone(&self) -> Frame {
        Frame {
            number: self.number,
        }
    }

    pub fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        FrameIter { start, end }
    }
}

pub struct FrameIter {
    start: Frame,
    end: Frame,
}
impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if self.start > self.end {
            return None;
        }
        let frame = self.start.clone();
        self.start.number += 1;
        Some(frame)
    }
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

pub struct AreaFrameAllocator {
    next_free_frame: Frame,
    current_area: Option<&'static MemoryArea>,
    memory_areas: &'static [MemoryArea],
    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
}
impl AreaFrameAllocator {
    pub fn new(
        kernel_start: usize,
        kernel_end: usize,
        multiboot_start: usize,
        multiboot_end: usize,
        memory_areas: &'static [MemoryArea],
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
        self.current_area = self
            .memory_areas
            .iter()
            .filter(|area| {
                let address = area.start_address() + area.size() - 1;
                Frame::containing_address(address as usize) >= self.next_free_frame
            })
            .min_by_key(|area| area.start_address());

        if let Some(area) = self.current_area {
            let start_frame = Frame::containing_address(area.start_address() as usize);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}
impl FrameAllocator for AreaFrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        while let Some(area) = self.current_area {
            let frame = Frame {
                number: self.next_free_frame.number,
            };
            let current_area_last_frame =
                Frame::containing_address(area.start_address() as usize + area.size() as usize - 1);

            if frame > current_area_last_frame {
                self.choose_next_area();
                continue;
            }

            if frame >= self.kernel_start && frame <= self.kernel_end
                || frame >= self.multiboot_start && frame <= self.multiboot_end
            {
                self.next_free_frame.number += 1;
                continue;
            }

            self.next_free_frame.number += 1;
            return Some(frame);
        }
        None
    }
    fn deallocate_frame(&mut self, _frame: Frame) {
        unimplemented!()
    }
}
