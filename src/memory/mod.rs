use spin::Mutex;
use heap_allocator::{HeapAllocator, LockedHeap};
use multiboot2::{BootInformation, BootInformationHeader};

mod heap_allocator;

#[global_allocator]
pub static HEAP_ALLOCATOR: LockedHeap = LockedHeap(Mutex::new(HeapAllocator::new()));

pub fn init(multiboot_addr: usize) {
    let kernel_end = unsafe {
        BootInformation::load(multiboot_addr as *const BootInformationHeader).unwrap().end_address()
    };

    let heap_start = crate::util::align_up(kernel_end, 4096);
    let heap_size = 1000 * 1024; // 1MiB
    HEAP_ALLOCATOR.lock().init(heap_start, heap_size);
}