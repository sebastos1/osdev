use crate::memory::{Page, ActivePageTable, PAGE_SIZE, FrameAllocator, PageIter, EntryFlags};

/// Manages fixed-size stack allocations within a page range.
pub struct StackAllocator {
    range: PageIter, // Range of pages available for stack allocation.
}

impl StackAllocator {
    /// Creates a stack allocator for a given range of pages.
    pub fn new(page_range: PageIter) -> StackAllocator {
        StackAllocator { range: page_range }
    }

    /// Allocates a stack of a specified size and returns it, if possible.
    /// Includes a guard page to prevent overflow.
    pub fn alloc_stack<FA: FrameAllocator>(
        &mut self,
        active_table: &mut ActivePageTable,
        frame_allocator: &mut FA,
        size_in_pages: usize,
    ) -> Option<Stack> {
        if size_in_pages == 0 { return None; } // Zero-sized stack is invalid.

        let mut range = self.range.clone();
        let guard_page = range.next(); // First page is a guard page.

        // Allocate stack pages based on size. Single-page stacks are handled differently.
        let stack_start = range.next();
        let stack_end = match size_in_pages {
            1 => stack_start,
            _ => range.nth(size_in_pages - 2), // For multi-page, find the end.
        };

        match (guard_page, stack_start, stack_end) {
            (Some(_), Some(start), end) => { // Ensure at least one page and a guard page can be allocated.
                self.range = range; // Update range on successful allocation.
                // Map stack pages to physical frames.
                let end_page = end.unwrap_or(start); // Single-page stack ends where it starts.
                for page in Page::range_inclusive(start, end_page) {
                    active_table.map(page, EntryFlags::WRITABLE, frame_allocator);
                }
                // Return the stack with proper top and bottom addresses.
                Some(Stack::new(end_page.start_address() + PAGE_SIZE, start.start_address()))
            }
            _ => None, // Insufficient pages for the requested stack size.
        }
    }
}

/// Represents a virtual memory stack with top and bottom addresses.
#[derive(Debug)]
#[allow(unused)]
pub struct Stack {
    top: usize,
    bottom: usize,
}

impl Stack {
    /// Constructs a new Stack with given top and bottom.
    fn new(top: usize, bottom: usize) -> Stack {
        assert!(top > bottom, "Stack top must be above bottom.");
        Stack { top, bottom }
    }

    /// Returns the top address of the stack.
    pub fn top(&self) -> usize { self.top }

    /// Returns the bottom address of the stack.
    #[allow(unused)]
    pub fn bottom(&self) -> usize { self.bottom }
}
