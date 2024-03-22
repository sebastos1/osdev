use spin::{Mutex, MutexGuard};
use core::alloc::{Layout, GlobalAlloc};

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct NodePointer(pub *mut Node);

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

    fn set_size(&self, size: usize) {
        unsafe { (*self.0).size = size }
    }

    fn next(&self) -> Option<NodePointer> {
        unsafe { (*self.0).next }
    }

    fn set_next(&self, next: Option<NodePointer>) {
        unsafe { (*self.0).next = next }
    }

    fn is_free(&self) -> bool {
        unsafe { (*self.0).is_free }
    }

    fn set_free(&self, is_free: bool) {
        unsafe { (*self.0).is_free = is_free }
    }
}

const NODE_SIZE: usize = core::mem::size_of::<Node>();

pub struct Node {
    pub size: usize,
    pub is_free: bool,
    pub next: Option<NodePointer>,
}

pub struct HeapAllocator {
    pub head: NodePointer, 
}

unsafe impl Send for HeapAllocator {}

impl HeapAllocator {
    pub const fn new() -> Self {
        Self {
            head: NodePointer(core::ptr::null_mut()),
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        let head_node = NodePointer(heap_start as *mut Node);
        head_node.set_next(None);
        head_node.set_free(true);
        head_node.set_size(heap_size);
        self.head = head_node;
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
        let mut current = Some(root.head);

        while let Some(free_node) = current {

            let alignment = 8;
            let inner_size = align_up(layout.size(), alignment);
            let outer_size = align_up(NODE_SIZE, alignment) + inner_size;
            // let padding = inner_size - layout.size(); something sus here

            if free_node.is_free() && free_node.size() >= outer_size {
                let new_node_addr = free_node.addr() + free_node.size() - outer_size;
                let mut return_pointer = new_node_addr;
                if free_node.size() > outer_size {
                    let new_node = NodePointer(new_node_addr as *mut Node);
                    new_node.set_next(Some(free_node));
                    new_node.set_free(false);
                    new_node.set_size(inner_size);

                    return_pointer = return_pointer + outer_size - inner_size;
                    free_node.set_size(free_node.size() - outer_size - alignment);
                    root.head = new_node;
                } else {
                    root.head = free_node.next().take().unwrap();
                }
                return return_pointer as *mut u8;
            }
            current = current.unwrap().next();
        }
        core::ptr::null_mut()
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let root = self.lock();
        let mut current = Some(root.head);
        
        while let Some(node) = current {
            if node.contains_addr(ptr as usize) {
                node.set_free(true);
                // no merging of nodes for now
                return;
            }
            current = node.next();
        }
        panic!("Attempted to deallocate memory that was not allocated!");
    }
}

pub fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}