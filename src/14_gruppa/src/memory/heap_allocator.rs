use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};

// A simple bump allocator for managing heap memory.
#[allow(unused)]
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: AtomicUsize,
}

impl BumpAllocator {
    // Creates a new `BumpAllocator` with a specified heap start and end.
    #[allow(unused)]
    pub const fn new(heap_start: usize, heap_end: usize) -> Self {
        Self {
            heap_start,
            heap_end,
            next: AtomicUsize::new(heap_start),
        }
    }
}

// Implementation of the GlobalAlloc trait for the `BumpAllocator`.
unsafe impl GlobalAlloc for BumpAllocator {
    // Allocates a block of memory.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        loop {
            // Load the current state of the `next` pointer.
            let current_next = self.next.load(Ordering::Relaxed);
            // Calculate the start address of the new allocation, aligned as required.
            let alloc_start = align_up(current_next, layout.align());
            // Calculate the end address of the new allocation.
            let alloc_end = alloc_start.saturating_add(layout.size());

            // Check if the end of the allocation is within the heap.
            if alloc_end <= self.heap_end {
                // Attempt to update the `next` pointer atomically.
                match self.next.compare_exchange(
                    current_next,
                    alloc_end,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return alloc_start as *mut u8, // Allocation succeeded.
                    Err(_) => continue, // Allocation failed; retry.
                }
            } else {
                // Heap is exhausted, cannot allocate.
                return null_mut();
            }
        }
    }

    // Deallocates a block of memory. (not supported by `BumpAllocator`)
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Deallocation is not supported in a bump allocator.
    }
}

// Aligns the given address upwards to the nearest multiple of `align`.
pub fn align_up(addr: usize, align: usize) -> usize {
    // Ensures `align` is a power of two.
    assert!(align.is_power_of_two(), "`align` must be a power of 2");
    (addr + align - 1) & !(align - 1)
}
