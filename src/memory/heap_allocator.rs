use crate::util::align_up;
use spin::{Mutex, MutexGuard};
use core::alloc::{Layout, GlobalAlloc};

const NODE_SIZE: usize = core::mem::size_of::<Node>();
const DEFAULT_ALIGNMENT: usize = 8;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct NodePointer(*mut Node);

impl NodePointer {
    fn addr(&self) -> usize {
        self.0 as usize
    }

    fn contains_addr(&self, addr: usize) -> bool {
        addr >= self.addr() && addr < self.addr() + self.size()
    }

    fn size(&self) -> usize {
        unsafe { (*self.0).size }
    }

    fn set_size(&self, size: usize) -> Self {
        unsafe { (*self.0).size = size }
        Self(self.0)
    }

    fn next(&self) -> Option<NodePointer> {
        unsafe { (*self.0).next }
    }

    fn set_next(&self, next: Option<NodePointer>) -> Self {
        unsafe { (*self.0).next = next }
        Self(self.0)
    }

    fn is_free(&self) -> bool {
        unsafe { (*self.0).is_free }
    }

    fn set_free(&self, is_free: bool) -> Self {
        unsafe { (*self.0).is_free = is_free }
        Self(self.0)
    }
}

struct Node {
    size: usize,
    is_free: bool,
    next: Option<NodePointer>,
}

pub struct HeapAllocator {
    head: Option<NodePointer>, 
}

unsafe impl Send for HeapAllocator {}

impl HeapAllocator {
    pub const fn new() -> Self {
        Self { head: None, }
    }

    pub fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.head = Some(NodePointer(heap_start as *mut Node)
            .set_next(None)
            .set_free(true)
            .set_size(heap_size));
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
        let mut root = self.lock();
        let mut current = root.head;

        while let Some(free_node) = current {

            let layout_align = if layout.align() > DEFAULT_ALIGNMENT {
                layout.align()
            } else {
                DEFAULT_ALIGNMENT
            };

            let inner_size = align_up(layout.size(), layout_align);
            let outer_size = align_up(NODE_SIZE, DEFAULT_ALIGNMENT) + inner_size;
            // let padding = inner_size - layout.size(); something sus here

            if free_node.is_free() && free_node.size() >= outer_size {
                let new_node_addr = free_node.addr() + free_node.size() - outer_size;
                let mut return_pointer = new_node_addr;
                if free_node.size() > outer_size {
                    let new_node = NodePointer(new_node_addr as *mut Node)
                        .set_next(Some(free_node))
                        .set_free(false)
                        .set_size(inner_size);

                    return_pointer = return_pointer + outer_size - inner_size;
                    free_node.set_size(free_node.size() - outer_size - DEFAULT_ALIGNMENT);
                    root.head = Some(new_node);
                } else {
                    root.head = free_node.next();
                }
                return return_pointer as *mut u8
            }
            current = free_node.next();
        }
        core::ptr::null_mut()
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let root = self.lock();
        let mut current = root.head;
        
        while let Some(node) = current {
            if node.contains_addr(ptr as usize) {
                node.set_free(true);
                return
            }
            current = node.next();
        }
        panic!("Allocation hallucination! Heap horror!");
    }
}