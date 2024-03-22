use spin::{Once, Mutex};
// use multiboot2::MemoryAreaType;
use heap_allocator::{HeapAllocator, LockedHeap};
use multiboot2::{BootInformation, BootInformationHeader};

mod heap_allocator;

pub static BOOT_INFO: Once<BootInformation> = Once::new();

#[global_allocator]
pub static HEAP_ALLOCATOR: LockedHeap = LockedHeap(Mutex::new(HeapAllocator::new()));

pub fn init(multiboot_addr: usize) {
    let boot_info = BOOT_INFO.call_once(||unsafe {
        BootInformation::load(multiboot_addr as *const BootInformationHeader).unwrap()
    });

    /*
    for region in boot_info.memory_map_tag().unwrap().memory_areas() {
        if region.typ() == MemoryAreaType::Available {
            println!("Available RAM: 0x{:x} - 0x{:x}", region.start_address(), region.end_address());
        }
    }

    let elf_sections = boot_info.elf_sections().unwrap();
    let _kernel_start = elf_sections.clone().map(|s| s.start_address()).min().unwrap();
    let _kernel_end = elf_sections.map(|s| s.start_address() + s.size()).max().unwrap();
    println!("kernel_start: 0x{:x}, kernel_end: 0x{:x}", kernel_start, kernel_end);

    let _bootinfo_start = boot_info.start_address(); */
    let bootinfo_end = boot_info.end_address();
    // println!("bootinfo_start: 0x{:x}, bootinfo_end: 0x{:x}", bootinfo_start, bootinfo_end);

    let heap_start = (bootinfo_end + 4096 - 1) & !(4096 - 1);
    let heap_size = 1000 * 1024; // 1MiB

    // println!("heap_end: 0x{:x}", heap_start + heap_size);

    unsafe {
        HEAP_ALLOCATOR.lock().init(heap_start, heap_size);
    }

    use alloc::vec::Vec;
    let vec1: Vec<u32> = (1..=91).collect();
    println!("vec: {:?}", vec1);

    // println!("head node: 0x{:x}", HEAP_ALLOCATOR.lock().head.0 as usize);
}