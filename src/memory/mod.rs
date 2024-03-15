use multiboot2::{BootInformation, BootInformationHeader};

mod frame_allocator;

pub const PAGE_SIZE: usize = 4096;

pub fn init(multiboot_addr: usize) {
    let boot_info = unsafe {
        BootInformation::load(multiboot_addr as *const BootInformationHeader).unwrap()
    };
    println!("{:#?}", boot_info);

    let memory_map_tag = boot_info.memory_map_tag().unwrap();

    for area in memory_map_tag.memory_areas() {
        println!("Available memory area: {:x?} - {:x?}", area.start_address(), area.end_address());
    }
}