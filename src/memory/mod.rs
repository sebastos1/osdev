use spin::{Once, Mutex};
use heap_allocator::{HeapAllocator, LockedHeap};
use multiboot2::{BootInformation, BootInformationHeader};

mod heap_allocator;

pub static BOOT_INFO: Once<BootInformation> = Once::new();

pub const HEAP_START: usize = 0x100000;
pub const HEAP_SIZE: usize = 2000 * 1024; // 1 MiB

#[global_allocator]
pub static HEAP_ALLOCATOR: LockedHeap = LockedHeap(Mutex::new(HeapAllocator::new()));

pub fn init(multiboot_addr: usize) {
    let boot_info = BOOT_INFO.call_once(||unsafe {
        BootInformation::load(multiboot_addr as *const BootInformationHeader).unwrap()
    });

    for region in boot_info.memory_map_tag().unwrap().memory_areas() {
        println!("start: 0x{:x}, length: 0x{:x}", region.start_address(), region.size());
    }

    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    use alloc::vec::Vec;
    let mut numbers: Vec<u32> = Vec::new();
    for i in 1..=512 {
        numbers.push(i);
    }
    println!("vec: {:?}", numbers);
}