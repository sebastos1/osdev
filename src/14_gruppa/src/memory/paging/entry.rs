use super::EntryFlags;
use crate::memory::Frame;

// Represents a single page table entry.
pub struct Entry(u64);

impl Entry {
    // Checks if the entry is not pointing to any frame (i.e., unused).
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    // Marks the entry as unused.
    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    // Retrieves the flags from the entry.
    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    // Returns the frame pointed to by this entry, if any.
    pub fn pointed_frame(&self) -> Option<Frame> {
        if self.flags().contains(EntryFlags::PRESENT) {
            Some(Frame::containing_address(self.0 as usize & 0x000fffff_fffff000))
        } else {
            None
        }
    }

    // Sets the entry to point to a given frame with specified flags.
    // Ensures that the frame's start address conforms to the expected address alignment.
    pub fn set(&mut self, frame: Frame, flags: EntryFlags) {
        assert!(frame.start_address() & !0x000fffff_fffff000 == 0, "Frame start address is not aligned.");
        self.0 = (frame.start_address() as u64) | flags.bits();
    }
}
