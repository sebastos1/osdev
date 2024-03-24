use crate::util::{align_up};
use spin::{Mutex, MutexGuard};
use core::mem::size_of;
use core::alloc::{Layout, GlobalAlloc};

const CHUNK: usize = 16; // 16 byte chunks
const NODE_SIZE_ALIGNED: usize = align_up(size_of::<Node>(), CHUNK);

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct NodePointer(*mut Node);

impl NodePointer {
    fn addr(&self) -> usize {
        self.0 as usize
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
            .set_size(heap_size - NODE_SIZE_ALIGNED)); // its aligned to 4096 already
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
        let root = self.lock();
        let mut current = root.head;

        let layout_size = align_up(layout.size(), CHUNK); // potential padding at the end
        let layout_offset = NODE_SIZE_ALIGNED;
        let total_size = layout_offset + layout_size;

        while let Some(free_node) = current {
            if free_node.is_free() && free_node.size() >= total_size {

                let new_node_addr = free_node.addr() + free_node.size() - total_size;
                let return_pointer = new_node_addr + layout_offset;

                NodePointer(new_node_addr as *mut Node)
                    .set_free(false)
                    .set_size(layout_size)
                    .set_next(None);
  
                free_node.set_size(free_node.size() - total_size); // might cause 0 sized nodes, but its ok

                return return_pointer as *mut u8
            }
            current = free_node.next();
        }
        core::ptr::null_mut()
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let mut root = self.lock();

        // if there's not a node here, we're in big trouble anyway. Just assume there is :)
        let node = NodePointer((ptr as usize - NODE_SIZE_ALIGNED) as *mut Node);
        
        if !node.is_free() {
            node.set_free(true);
            node.set_next(root.head);
            root.head = Some(node);
            return
        }
        panic!("Failed to deallocate memory");      
    }
}