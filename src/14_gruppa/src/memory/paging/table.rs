use core::marker::PhantomData;
use core::ops::{Index, IndexMut};
use super::{entry::Entry, EntryFlags};
use crate::memory::{FrameAllocator, ENTRY_COUNT};

// The top-level P4 table's fixed virtual address in a typical x86_64 system.
pub const P4: *mut Table<Level4> = 0xffff_ffff_ffff_f000 as *mut _;

// A generic page table structure for any level (P4, P3, P2, P1).
pub struct Table<L: TableLevel> {
    entries: [Entry; ENTRY_COUNT], // Page table entries
    level: PhantomData<L>, // Marker to indicate the table level, does not occupy space
}

// Common methods applicable to all table levels
impl<L: TableLevel> Table<L> {
    // Clears all entries in the table
    pub fn zero(&mut self) {
        for entry in &mut self.entries {
            entry.set_unused();
        }
    }
}

// Methods specific to hierarchical levels (P4 to P1), excluding the final level (P1)
impl<L: HierarchicalLevel> Table<L> {
    // Calculates the address of the next-level table based on the current entry, if present
    fn next_table_address(&self, index: usize) -> Option<usize> {
        let entry_flags = self[index].flags();
        if entry_flags.contains(EntryFlags::PRESENT) && !entry_flags.contains(EntryFlags::HUGE_PAGE)
        {
            let table_address = self as *const _ as usize;
            Some((table_address << 9) | (index << 12))
        } else {
            None
        }
    }

    // Gets a reference to the next-level table, if present
    pub fn next_table(&self, index: usize) -> Option<&Table<L::NextLevel>> {
        self.next_table_address(index)
            .map(|address| unsafe { &*(address as *const _) })
    }

    // Gets a mutable reference to the next-level table, if present
    pub fn next_table_mut(&mut self, index: usize) -> Option<&mut Table<L::NextLevel>> {
        self.next_table_address(index)
            .map(|address| unsafe { &mut *(address as *mut _) })
    }

    // Ensures the next-level table exists, creating it if necessary
    pub fn next_table_create<A>(&mut self, index: usize, allocator: &mut A) -> &mut Table<L::NextLevel>
    where
        A: FrameAllocator,
    {
        if self.next_table(index).is_none() {
            assert!(
                !self.entries[index].flags().contains(EntryFlags::HUGE_PAGE),
                "mapping code does not support huge pages"
            );
            let frame = allocator.allocate_frame().expect("no frames available");
            self.entries[index].set(frame, EntryFlags::PRESENT | EntryFlags::WRITABLE);
            self.next_table_mut(index).unwrap().zero();
        }
        self.next_table_mut(index).unwrap()
    }
}

// Allows indexing into the table's entries array
impl<L: TableLevel> Index<usize> for Table<L> {
    type Output = Entry;
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

// Allows mutable indexing into the table's entries array
impl<L: TableLevel> IndexMut<usize> for Table<L> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

// Marker traits to indicate the level of a table
pub trait TableLevel {}
pub enum Level4 {}
pub enum Level3 {}
pub enum Level2 {}
pub enum Level1 {}
impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}

// A trait to define a relationship between table levels
pub trait HierarchicalLevel: TableLevel {
    type NextLevel: TableLevel;
}
impl HierarchicalLevel for Level4 { type NextLevel = Level3; }
impl HierarchicalLevel for Level3 { type NextLevel = Level2; }
impl HierarchicalLevel for Level2 { type NextLevel = Level1; }
