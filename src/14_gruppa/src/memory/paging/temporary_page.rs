use super::{table::{Level1, Table}, EntryFlags};
use crate::memory::{ActivePageTable, Frame, FrameAllocator, Page, VirtualAddress};

// Represents a temporary page that can be mapped to a frame.
pub struct TemporaryPage {
    page: Page,
    allocator: TinyAllocator,
}

impl TemporaryPage {
    // Creates a new TemporaryPage with a given page and a reference to a FrameAllocator.
    pub fn new<A>(page: Page, allocator: &mut A) -> Self
    where
        A: FrameAllocator,
    {
        TemporaryPage {
            page,
            allocator: TinyAllocator::new(allocator),
        }
    }

    // Maps the temporary page to a frame, making it accessible.
    pub fn map(&mut self, frame: Frame, active_table: &mut ActivePageTable) -> VirtualAddress {
        assert!(active_table.translate_page(self.page).is_none(), "temporary page is already mapped");
        active_table.map_to(self.page, frame, EntryFlags::WRITABLE, &mut self.allocator);
        self.page.start_address()
    }

    // Unmaps the temporary page, making the frame inaccessible.
    pub fn unmap(&mut self, active_table: &mut ActivePageTable) {
        active_table.unmap(self.page, &mut self.allocator)
    }

    // Maps a table frame to the temporary page and returns a mutable reference to the table.
    pub fn map_table_frame(&mut self, frame: Frame, active_table: &mut ActivePageTable) -> &mut Table<Level1> {
        unsafe { &mut *(self.map(frame, active_table) as *mut Table<Level1>) }
    }
}

// A minimal frame allocator that can allocate up to three frames.
struct TinyAllocator([Option<Frame>; 3]);

impl TinyAllocator {
    // Initializes the TinyAllocator with up to three frames from the given FrameAllocator.
    fn new<A>(allocator: &mut A) -> Self
    where
        A: FrameAllocator,
    {
        let mut allocate = || allocator.allocate_frame();
        let frames = [allocate(), allocate(), allocate()];
        TinyAllocator(frames)
    }
}

impl FrameAllocator for TinyAllocator {
    // Allocates a frame if available.
    fn allocate_frame(&mut self) -> Option<Frame> {
        self.0.iter_mut().find_map(|frame| frame.take())
    }

    // Deallocates a frame, making it available again.
    fn deallocate_frame(&mut self, frame: Frame) {
        if let Some(slot) = self.0.iter_mut().find(|frame| frame.is_none()) {
            *slot = Some(frame);
        } else {
            panic!("Tiny allocator can hold only 3 frames.");
        }
    }
}
