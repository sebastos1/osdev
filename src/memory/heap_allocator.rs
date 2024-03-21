use spin::{Mutex, MutexGuard};
use core::alloc::{Layout, GlobalAlloc};

pub struct HeapAllocator {
    pub bottom: usize,
    pub size: usize,
    pub top: usize,
}

impl HeapAllocator {
    pub const fn new() -> Self {
        Self {
            bottom: 0,
            size: 0,
            top: 0,
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.bottom = heap_start;
        self.size = heap_size;
        self.top = self.bottom;
    }
}

pub struct LockedHeap(pub Mutex<HeapAllocator>); // dapper wrapper

impl LockedHeap {
    pub fn lock(&self) -> MutexGuard<HeapAllocator> {
        self.0.lock()
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut alloc = self.lock();
        if alloc.top.saturating_add(layout.size()) > alloc.bottom.saturating_add(alloc.size) {
            return core::ptr::null_mut();
        }
        let result = alloc.top;
        alloc.top = alloc.top.saturating_add(layout.size());
        
        core::ptr::write_bytes(result as *mut u8, 0, layout.size());

        result as *mut u8
    } 
    
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // unimplemented!(); no-op in a simple heap allocator
    }
}